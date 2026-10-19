use crate::pack::readers::{ManifestSource, SideAssets};
use crate::pack::writers::chunker::{SideChunks, chunk_files};
use crate::pack::{
    Certificate, ChunkEntry, CommonHeader, FileEntry, FileLocation, Handle, IndexRef, IndexV1,
    MagicValue, ManifestV1, Purpose, SignatureV1, Signed, TBPK_CURRENT_VERSION, TableOfContents,
    Target, TbManifest, TbPack, TbSignature,
};
use crate::pack::{
    DEFAULT_CHUNK_SIZE, HEADER_LEN, INDEX, MAX_CHUNK_SIZE, MAX_MANIFEST_LEN, MAX_SIGNATURE_LEN,
    MIN_CHUNK_SIZE, TOC_LEN, ZSTD_LEVEL, pack_digest,
};
use ed25519_dalek::{Signer, SigningKey};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
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
        info!(
            "Created pack signer with public key {}",
            hex(key.verifying_key().as_bytes())
        );
        Ok(Self { key, cert })
    }

    /// signs exactly the bytes that get written, never a re-encoding of `parsed`
    pub fn sign(&self, manifest: &Signed<TbManifest>) -> SignatureV1 {
        let digest = pack_digest(&manifest.raw);
        let signature = self.key.sign(&digest).to_bytes();
        info!(
            "Signed {} manifest bytes, digest {}",
            manifest.raw.len(),
            hex(&digest)
        );
        SignatureV1::new(self.cert.clone(), signature)
    }
}

pub struct PackWriter {
    // implemented
    tbpk: Option<TbPack>,
    // implemented
    chunk_size: u32,
    /// zstd level chunks are compressed at. defaults to the manifest version's pinned level
    zstd_level: Option<i32>,
    /// each side's chunks, indexed by `Target as usize`. filled by `build_tbpk`
    chunks: [Option<SideChunks>; 2],
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
            zstd_level: Some(ZSTD_LEVEL),
            chunks: [None, None],
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
        info!("Pack writer will sign packs");
        self.signer = Some(signer);
        self
    }

    /// overrides the zstd level, `None` stores every chunk uncompressed. anything other than
    /// `ZSTD_LEVEL` makes chunk bytes and hashes differ from what the manifest version pins,
    /// so it's for testing speed and size
    pub fn with_zstd_level(mut self, level: Option<i32>) -> Result<Self, String> {
        match level {
            Some(level) => {
                let range = zstd::compression_level_range();
                if !range.contains(&level) {
                    return Err(format!(
                        "zstd level {level} is out of range {}..={}",
                        range.start(),
                        range.end()
                    ));
                }
                if level != ZSTD_LEVEL {
                    tracing::warn!(
                        "Pack writer zstd level is {level}, not the pinned {ZSTD_LEVEL}"
                    );
                }
                info!("Pack writer will compress chunks at zstd level {level}");
            }
            None => {
                tracing::warn!("Pack writer compression is off, not the pinned {ZSTD_LEVEL}");
            }
        }
        self.zstd_level = level;
        Ok(self)
    }

    pub fn load_manifest(&mut self, path: PathBuf) -> Result<(), String> {
        info!("Loading manifest from {}", path.display());
        let src = ManifestSource::from_path(&path)?;
        info!("Asset paths are relative to {}", src.get_root().display());
        self.manifest_source = Some(src);
        Ok(())
    }

    fn header(&self, body_length: u64, target: Target) -> CommonHeader {
        CommonHeader::new(MagicValue::TBPK, TBPK_CURRENT_VERSION, body_length)
    }

    pub fn get_manifest_source(&self) -> Result<&ManifestSource, String> {
        self.manifest_source
            .as_ref()
            .ok_or_else(|| "no manifest has been loaded, yet".into())
    }

    /// reads and compresses every file on one side
    fn chunk(&mut self, target: Target) -> Result<(), String> {
        info!("Chunking {target:?} files");
        let source = self.get_manifest_source()?;
        let files = source.get_side_files(target)?;
        let chunks = chunk_files(source.get_root(), &files, self.chunk_size, self.zstd_level)?;

        info!(
            "Chunked {} files into {} chunks for {target:?}",
            files.len(),
            chunks.entries.len()
        );
        self.chunks[target as usize] = Some(chunks);
        Ok(())
    }

    fn get_chunks(&self, target: Target) -> Result<&SideChunks, String> {
        self.chunks[target as usize]
            .as_ref()
            .ok_or_else(|| format!("{target:?} files haven't been chunked, yet"))
    }

    fn get_index(&self, target: Target) -> Result<IndexV1, String> {
        let chunks = self.get_chunks(target)?;
        let index = IndexV1 {
            chunk_size: self.chunk_size,
            chunks: chunks.entries.clone(),
            files: chunks.locations.clone(),
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

        let data_offset = index_offset + index_len;
        let data_len = self.get_chunks(target)?.data_len();

        info!("{target:?} manifest section at {manifest_offset}, {manifest_len} bytes");
        info!("{target:?} signature section at {signature_offset}, {signature_len} bytes");
        info!("{target:?} index section at {index_offset}, {index_len} bytes");
        info!("{target:?} data section at {data_offset}, {data_len} bytes");

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
        let source = self.get_manifest_source()?;

        let index = self.encode_index(target)?;
        let index_ref = IndexRef {
            len: index.len() as u64,
            hash: blake3::derive_key(INDEX, &index),
        };
        info!(
            "{target:?} index is {} bytes, hash {}",
            index_ref.len,
            hex(&index_ref.hash)
        );

        let manifest = source.get_v1_side(target, index_ref)?;
        info!(
            "Built {target:?} manifest with {} files, {} assets, {} scripts",
            manifest.files.len(),
            manifest.assets.len(),
            manifest.scripts.len()
        );
        Ok(manifest)
    }

    fn get_signed_manifest(&self, target: Target) -> Result<Signed<TbManifest>, String> {
        let manifest = TbManifest::encode(self.get_manifest(target)?)?;
        info!(
            "Encoded {target:?} manifest to {} bytes",
            manifest.raw.len()
        );
        Ok(manifest)
    }

    fn get_signature(&self, manifest: &Signed<TbManifest>) -> Option<SignatureV1> {
        self.signer.as_ref().map(|signer| signer.sign(manifest))
    }

    fn get_tb_signature(
        &self,
        manifest: &Signed<TbManifest>,
    ) -> Result<Option<TbSignature>, String> {
        if self.signer.is_none() {
            info!("No signer set, the pack will be unsigned");
        }
        self.get_signature(manifest)
            .map(TbSignature::new)
            .transpose()
    }

    pub fn build_tbpk(&mut self, target: Target) -> Result<(), String> {
        info!("Building {target:?} pack");
        // the index comes from the chunks, and the manifest holds the index's hash
        self.chunk(target)?;

        // the signature covers the manifest's bytes, so the manifest has to exist first
        let manifest = self.get_signed_manifest(target)?;
        let signature = self.get_tb_signature(&manifest)?;
        let toc = self.get_table_of_contents(target, &manifest, signature.as_ref())?;

        // the data section is last, so its end is the end of the file
        let body_length = toc.data_offset + toc.data_len - HEADER_LEN as u64;

        let tbpk = TbPack {
            header: self.header(body_length, target),
            toc,
            index: self.get_index(target)?,
            manifest,
            signature,
        };

        self.tbpk = Some(tbpk);

        info!(
            "Built {target:?} pack, {} bytes total",
            HEADER_LEN as u64 + body_length
        );
        Ok(())
    }

    /// writes the built pack to `path`, sections in toc order with no gaps
    pub fn write_tbpk_out(&self, path: &Path) -> Result<(), String> {
        let tbpk_value = match self.tbpk.as_ref() {
            None => return Err("TBPK must be built before calling write out".into()),
            Some(v) => v,
        };
        let toc = &tbpk_value.toc;
        let target = tbpk_value.manifest.parsed.body.target;
        let chunks = self.get_chunks(target)?;
        info!("Writing {target:?} pack to {}", path.display());

        let index = postcard::to_allocvec(&tbpk_value.index).map_err(|e| e.to_string())?;
        if index.len() as u64 != toc.index_len {
            return Err("index length doesn't match the table of contents".into());
        }

        let signature = match tbpk_value.signature.as_ref() {
            None => Vec::new(),
            Some(sig) => sig.to_bytes()?,
        };

        let file = File::create(path).map_err(|e| format!("`{}`: {e}", path.display()))?;
        let mut out = BufWriter::new(file);
        for (name, section) in [
            ("header", &tbpk_value.header.to_bytes()[..]),
            ("table of contents", &toc.to_bytes()),
            ("manifest", &tbpk_value.manifest.raw),
            ("signature", &signature),
            ("index", &index),
        ] {
            out.write_all(section).map_err(|e| e.to_string())?;
            info!("Wrote {name} section, {} bytes", section.len());
        }

        let data_len =
            std::io::copy(&mut chunks.data.open()?, &mut out).map_err(|e| e.to_string())?;
        if data_len != toc.data_len {
            return Err("data section length doesn't match the table of contents".into());
        }
        info!("Wrote data section, {data_len} bytes");
        out.flush().map_err(|e| e.to_string())?;

        info!("Wrote tbpk to {}", path.display());
        Ok(())
    }
}

/// lowercase hex, for logging hashes and keys
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
