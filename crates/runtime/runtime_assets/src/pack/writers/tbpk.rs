use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{
    Certificate, ChunkEntry, CommonHeader, FileEntry, FileLocation, Handle, IndexV1, MagicValue,
    ManifestV1, Purpose, SignatureV1, Signed, TBPK_CURRENT_VERSION, TableOfContents, Target,
    TbManifest, TbPack, TbSignature,
};
use crate::pack::{DEFAULT_CHUNK_SIZE, MAX_CHUNK_SIZE, MIN_CHUNK_SIZE, pack_digest};
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

    fn get_table_of_contents(&self, target: Target) -> Result<TableOfContents, String> {}

    fn get_manifest(&self, target: Target) -> Option<ManifestV1> {}

    fn get_signed_manifest(&self, target: Target) -> Result<Signed<TbManifest>, String> {
        let manifest = match self.get_manifest(target) {
            None => return Err("no manifest exists, yet".into()),
            Some(v) => v,
        };
        TbManifest::encode(manifest)
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
            toc: self.get_table_of_contents(target)?,
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
