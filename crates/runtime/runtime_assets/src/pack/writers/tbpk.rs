use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{
    Certificate, ChunkEntry, CommonHeader, FileEntry, FileLocation, Handle, IndexRef, IndexV1,
    MagicValue, ManifestV1, Purpose, SignatureV1, Signed, TBPK_CURRENT_VERSION, TableOfContents,
    Target, TbManifest, TbPack, TbSignature,
};
use crate::pack::{
    DEFAULT_CHUNK_SIZE, HEADER_LEN, INDEX, MAX_CHUNK_SIZE, MAX_MANIFEST_LEN, MAX_SIGNATURE_LEN,
    MIN_CHUNK_SIZE, TOC_LEN, pack_digest,
};
use ed25519_dalek::{Signer, SigningKey};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::info;

/// a pack signing key and the certificate the root key issued for it
pub struct PackSigner {
    key: SigningKey,
    cert: Certificate,
}

impl PackSigner {
    /// fails if the certificate isn't for packs or doesn't match the key
    pub fn new(key: SigningKey, cert: Certificate) -> Result<Self, String> {
        if cert.purpose() != Purpose::Pack {
            return Err(format!(
                "certificate purpose is {:?}, not Pack",
                cert.purpose()
            ));
        }
        if cert.public_key() != key.verifying_key().as_bytes() {
            return Err("certificate public key doesn't match the signing key".into());
        }
        Ok(Self { key, cert })
    }

    /// signs exactly the bytes that get written, never a re-encoding of `parsed`
    pub fn sign(&self, manifest: &Signed<TbManifest>) -> SignatureV1 {
        let digest = pack_digest(&manifest.raw);
        let signature = self.key.sign(&digest).to_bytes();
        SignatureV1::new(self.cert.clone(), signature)
    }
}

pub struct PackWriter {
    // implemented
    tbpk: Option<TbPack>,
    // implemented
    chunk_size: u32,
    // needs implementation
    chunk_entries: Vec<ChunkEntry>,
    // needs implementation
    file_locations: Vec<FileLocation>,
    /// `None` writes an unsigned pack
    signer: Option<PackSigner>,
    // implemented
    manifest_source: Option<ManifestSource>,
}

impl Default for PackWriter {
    fn default() -> Self {
        Self {
            tbpk: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunk_entries: Vec::new(),
            file_locations: Vec::new(),
            signer: None,
            manifest_source: None,
        }
    }
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_signer(mut self, signer: PackSigner) -> Self {
        self.signer = Some(signer);
        self
    }

    pub fn load_manifest(&mut self, path: PathBuf) -> Result<(), String> {
        let src = ManifestSource::from_path(&path)?;
        self.manifest_source = Some(src);
        Ok(())
    }

    fn header(&self, body_length: u64, target: Target) -> CommonHeader {
        CommonHeader::new(MagicValue::TBPK, TBPK_CURRENT_VERSION, body_length)
    }

    fn get_chunk_entries(&self, target: Target) -> &Vec<ChunkEntry> {
        &self.chunk_entries
    }

    fn get_file_locations(&self, target: Target) -> &Vec<FileLocation> {
        &self.file_locations
    }

    fn get_index(&self, target: Target) -> Result<IndexV1, String> {
        let index = IndexV1 {
            chunk_size: self.chunk_size,
            chunks: self.get_chunk_entries(target).clone(),
            files: self.get_file_locations(target).clone(),
        };

        Ok(index)
    }

    /// the index section's bytes. the manifest's index hash is over exactly these
    fn encode_index(&self, target: Target) -> Result<Vec<u8>, String> {
        postcard::to_allocvec(&self.get_index(target)?).map_err(|e| e.to_string())
    }

    /// sections go header, toc, manifest, signature, index, data, with no gaps
    fn get_table_of_contents(
        &self,
        target: Target,
        manifest: &Signed<TbManifest>,
        signature: Option<&TbSignature>,
    ) -> Result<TableOfContents, String> {
        let manifest_offset = HEADER_LEN as u64 + TOC_LEN;
        let manifest_len = manifest.raw.len() as u64;
        if manifest_len > MAX_MANIFEST_LEN {
            return Err(format!(
                "manifest is {manifest_len} bytes, the most is {MAX_MANIFEST_LEN}"
            ));
        }

        let signature_offset = manifest_offset + manifest_len;
        let signature_len = signature.map_or(0, |sig| HEADER_LEN as u64 + sig.header().body_length);
        if signature_len > MAX_SIGNATURE_LEN {
            return Err(format!(
                "signature is {signature_len} bytes, the most is {MAX_SIGNATURE_LEN}"
            ));
        }

        let index_offset = signature_offset + signature_len;
        let index_len = self.encode_index(target)?.len() as u64;

        // chunks are in data section order, so the last one ends the section
        let data_offset = index_offset + index_len;
        let data_len = self
            .get_chunk_entries(target)
            .last()
            .map_or(0, |chunk| chunk.offset + chunk.stored_len as u64);

        Ok(TableOfContents {
            manifest_offset,
            manifest_len,
            signature_offset,
            signature_len,
            index_offset,
            index_len,
            data_offset,
            data_len,
        })
    }

    /// needs the index built first, since the manifest holds its length and hash
    fn get_manifest(&self, target: Target) -> Result<ManifestV1, String> {
        let source = self
            .manifest_source
            .as_ref()
            .ok_or("no manifest has been loaded, yet")?;

        let index = self.encode_index(target)?;
        let index_ref = IndexRef {
            len: index.len() as u64,
            hash: blake3::derive_key(INDEX, &index),
        };

        source.get_v1_side(target, index_ref)
    }

    fn get_signed_manifest(&self, target: Target) -> Result<Signed<TbManifest>, String> {
        TbManifest::encode(self.get_manifest(target)?)
    }

    fn get_signature(&self, manifest: &Signed<TbManifest>) -> Option<SignatureV1> {
        self.signer.as_ref().map(|signer| signer.sign(manifest))
    }

    fn get_tb_signature(
        &self,
        manifest: &Signed<TbManifest>,
    ) -> Result<Option<TbSignature>, String> {
        self.get_signature(manifest)
            .map(TbSignature::new)
            .transpose()
    }

    pub fn build_tbpk(&mut self, target: Target) -> Result<(), String> {
        let body_length = 0;

        // the signature covers the manifest's bytes, so the manifest has to exist first
        let manifest = self.get_signed_manifest(target)?;
        let signature = self.get_tb_signature(&manifest)?;

        let tbpk = TbPack {
            header: self.header(body_length, target),
            toc: self.get_table_of_contents(target, &manifest, signature.as_ref())?,
            index: self.get_index(target)?,
            manifest,
            signature,
        };

        self.tbpk = Some(tbpk);

        Ok(())
    }

    pub fn write_tbpk_out(&self) -> Result<(), String> {
        let tbpk_value = match self.tbpk.as_ref() {
            None => return Err("TBPK must be built before calling write out".into()),
            Some(v) => v,
        };

        Ok(())
    }
}
