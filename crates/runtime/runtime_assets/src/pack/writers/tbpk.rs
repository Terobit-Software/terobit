use crate::pack::writers::chunker::{ChunkError, SideChunks, chunk_files};
use crate::pack::{
    Certificate, ChunkEntry, CommonHeader, EncodeError, FileEntry, FileLocation, Handle, IndexRef,
    IndexV1, MagicValue, ManifestV1, Purpose, SignatureV1, Signed, TBPK_CURRENT_VERSION,
    TableOfContents, Target, TbManifest, TbPack, TbSignature,
};
use crate::pack::{
    DEFAULT_CHUNK_SIZE, HEADER_LEN, INDEX, MAX_CHUNK_SIZE, MAX_MANIFEST_LEN, MAX_SIGNATURE_LEN,
    MIN_CHUNK_SIZE, TOC_LEN, ZSTD_LEVEL, pack_digest,
};
use crate::{ManifestLoadError, ManifestSideError, ManifestSource, SideAssets};
use ed25519_dalek::{Signer, SigningKey};
use snafu::{OptionExt, ResultExt, Snafu, ensure};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// a signing key and certificate that can't be used together to sign packs
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum PackSignerError {
    #[snafu(display(
        "certificate is for {purpose:?} signing, but pack signing needs a {:?} certificate",
        Purpose::Pack
    ))]
    WrongPurpose { purpose: Purpose },

    #[snafu(display(
        "certificate public key {cert_key} doesn't match the signing key's public key {signing_key}, the certificate was issued for a different key"
    ))]
    KeyMismatch {
        cert_key: String,
        signing_key: String,
    },
}

/// a pack couldn't be configured, built or written
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum PackWriterError {
    #[snafu(display("zstd level {level} is out of the range zstd supports, {min}..={max}"))]
    ZstdLevelOutOfRange { level: i32, min: i32, max: i32 },

    #[snafu(display("pack writer couldn't load its source manifest"))]
    LoadManifest {
        #[snafu(source(from(ManifestLoadError, Box::new)))]
        source: Box<ManifestLoadError>,
    },

    #[snafu(display(
        "no manifest has been loaded yet, call `load_manifest` before building a pack"
    ))]
    NoManifest,

    #[snafu(display("failed to list the {target:?} files to pack"))]
    SideFiles {
        target: Target,
        #[snafu(source(from(ManifestSideError, Box::new)))]
        source: Box<ManifestSideError>,
    },

    #[snafu(display("failed to chunk the {target:?} files from `{}`", root.display()))]
    Chunk {
        target: Target,
        root: PathBuf,
        #[snafu(source(from(ChunkError, Box::new)))]
        source: Box<ChunkError>,
    },

    #[snafu(display(
        "{target:?} files haven't been chunked yet, `build_tbpk` chunks them before anything reads the chunks"
    ))]
    NotChunked { target: Target },

    #[snafu(display("failed to postcard encode the {target:?} index section"))]
    EncodeIndex {
        target: Target,
        source: postcard::Error,
    },

    #[snafu(display("failed to build the {target:?} manifest"))]
    BuildManifest {
        target: Target,
        #[snafu(source(from(ManifestSideError, Box::new)))]
        source: Box<ManifestSideError>,
    },

    #[snafu(display("failed to encode the {target:?} manifest section"))]
    EncodeManifest {
        target: Target,
        #[snafu(source(from(EncodeError, Box::new)))]
        source: Box<EncodeError>,
    },

    #[snafu(display("failed to encode the {target:?} signature section"))]
    EncodeSignature {
        target: Target,
        #[snafu(source(from(EncodeError, Box::new)))]
        source: Box<EncodeError>,
    },

    #[snafu(display(
        "{target:?} manifest section is {len} bytes, but the most a pack allows is {MAX_MANIFEST_LEN}"
    ))]
    ManifestTooLarge { target: Target, len: u64 },

    #[snafu(display(
        "{target:?} signature section is {len} bytes, but the most a pack allows is {MAX_SIGNATURE_LEN}"
    ))]
    SignatureTooLarge { target: Target, len: u64 },

    #[snafu(display(
        "no pack has been built yet, call `build_tbpk` before `write_tbpk_out` (writing to `{}`)",
        path.display()
    ))]
    NotBuilt { path: PathBuf },

    #[snafu(display(
        "{target:?} index section encoded to {actual} bytes, but the table of contents says {expected}, the pack would be corrupt"
    ))]
    IndexLenMismatch {
        target: Target,
        actual: u64,
        expected: u64,
    },

    #[snafu(display("failed to create pack file `{}`", path.display()))]
    CreateOutput {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "failed to write the {section} section ({len} bytes) to `{}`",
        path.display()
    ))]
    WriteSection {
        section: &'static str,
        len: usize,
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("failed to open the chunk spool file `{}` to copy the data section", spool.display()))]
    OpenSpool {
        spool: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "failed to copy the data section from spool file `{}` to `{}`",
        spool.display(),
        path.display()
    ))]
    CopyData {
        spool: PathBuf,
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display(
        "{target:?} data section copied {actual} bytes, but the table of contents says {expected}, the pack would be corrupt"
    ))]
    DataLenMismatch {
        target: Target,
        actual: u64,
        expected: u64,
    },

    #[snafu(display("failed to flush pack file `{}`", path.display()))]
    FlushOutput {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// a pack signing key and the certificate the root key issued for it
pub struct PackSigner {
    key: SigningKey,
    cert: Certificate,
}

impl PackSigner {
    /// fails if the certificate isn't for packs or doesn't match the key
    pub fn new(key: SigningKey, cert: Certificate) -> Result<Self, PackSignerError> {
        ensure!(
            cert.purpose() == Purpose::Pack,
            WrongPurposeSnafu {
                purpose: cert.purpose()
            }
        );
        ensure!(
            cert.public_key() == key.verifying_key().as_bytes(),
            KeyMismatchSnafu {
                cert_key: hex(cert.public_key()),
                signing_key: hex(key.verifying_key().as_bytes()),
            }
        );
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
    pub fn with_zstd_level(mut self, level: Option<i32>) -> Result<Self, PackWriterError> {
        match level {
            Some(level) => {
                let range = zstd::compression_level_range();
                ensure!(
                    range.contains(&level),
                    ZstdLevelOutOfRangeSnafu {
                        level,
                        min: *range.start(),
                        max: *range.end()
                    }
                );
                if level != ZSTD_LEVEL {
                    warn!(
                        "Pack writer zstd level is {level}, not the {ZSTD_LEVEL} pinned by manifest v1. \
                         Chunk bytes and hashes will differ from a release build of the same content, \
                         only use this to test speed and size"
                    );
                }
                info!("Pack writer will compress chunks at zstd level {level}");
            }
            None => {
                warn!(
                    "Pack writer compression is off, every chunk is stored raw instead of at the \
                     zstd level {ZSTD_LEVEL} pinned by manifest v1. Packs will be bigger and chunk \
                     hashes will differ from a release build, only use this to test speed and size"
                );
            }
        }
        self.zstd_level = level;
        Ok(self)
    }

    pub fn load_manifest(&mut self, path: PathBuf) -> Result<(), PackWriterError> {
        info!("Loading manifest from {}", path.display());
        let src = ManifestSource::from_path(&path).context(LoadManifestSnafu)?;
        info!("Asset paths are relative to {}", src.get_root().display());
        self.manifest_source = Some(src);
        Ok(())
    }

    fn header(&self, body_length: u64, target: Target) -> CommonHeader {
        CommonHeader::new(MagicValue::TBPK, TBPK_CURRENT_VERSION, body_length)
    }

    pub fn get_manifest_source(&self) -> Result<&ManifestSource, PackWriterError> {
        self.manifest_source.as_ref().context(NoManifestSnafu)
    }

    /// reads and compresses every file on one side
    fn chunk(&mut self, target: Target) -> Result<(), PackWriterError> {
        info!("Chunking {target:?} files");
        let source = self.get_manifest_source()?;
        let files = source
            .get_side_files(target)
            .context(SideFilesSnafu { target })?;
        let root = source.get_root();
        let chunks = chunk_files(root, &files, self.chunk_size, self.zstd_level)
            .context(ChunkSnafu { target, root })?;

        info!(
            "Chunked {} files into {} chunks for {target:?}",
            files.len(),
            chunks.entries.len()
        );
        self.chunks[target as usize] = Some(chunks);
        Ok(())
    }

    fn get_chunks(&self, target: Target) -> Result<&SideChunks, PackWriterError> {
        self.chunks[target as usize]
            .as_ref()
            .context(NotChunkedSnafu { target })
    }

    fn get_index(&self, target: Target) -> Result<IndexV1, PackWriterError> {
        let chunks = self.get_chunks(target)?;
        let index = IndexV1 {
            chunk_size: self.chunk_size,
            chunks: chunks.entries.clone(),
            files: chunks.locations.clone(),
        };

        Ok(index)
    }

    /// the index section's bytes. the manifest's index hash is over exactly these
    fn encode_index(&self, target: Target) -> Result<Vec<u8>, PackWriterError> {
        postcard::to_allocvec(&self.get_index(target)?).context(EncodeIndexSnafu { target })
    }

    /// sections go header, toc, manifest, signature, index, data, with no gaps
    fn get_table_of_contents(
        &self,
        target: Target,
        manifest: &Signed<TbManifest>,
        signature: Option<&TbSignature>,
    ) -> Result<TableOfContents, PackWriterError> {
        let manifest_offset = HEADER_LEN as u64 + TOC_LEN;
        let manifest_len = manifest.raw.len() as u64;
        ensure!(
            manifest_len <= MAX_MANIFEST_LEN,
            ManifestTooLargeSnafu {
                target,
                len: manifest_len
            }
        );

        let signature_offset = manifest_offset + manifest_len;
        let signature_len = signature.map_or(0, |sig| HEADER_LEN as u64 + sig.header().body_length);
        ensure!(
            signature_len <= MAX_SIGNATURE_LEN,
            SignatureTooLargeSnafu {
                target,
                len: signature_len
            }
        );

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
    fn get_manifest(&self, target: Target) -> Result<ManifestV1, PackWriterError> {
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

        let manifest = source
            .get_v1_side(target, index_ref)
            .context(BuildManifestSnafu { target })?;
        info!(
            "Built {target:?} manifest with {} files, {} assets, {} scripts",
            manifest.files.len(),
            manifest.assets.len(),
            manifest.scripts.len()
        );
        Ok(manifest)
    }

    fn get_signed_manifest(&self, target: Target) -> Result<Signed<TbManifest>, PackWriterError> {
        let manifest = TbManifest::encode(self.get_manifest(target)?)
            .context(EncodeManifestSnafu { target })?;
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
        target: Target,
        manifest: &Signed<TbManifest>,
    ) -> Result<Option<TbSignature>, PackWriterError> {
        if self.signer.is_none() {
            warn!(
                "No signer set with `with_signer`, the {target:?} pack will be written without a \
                 signature section"
            );
        }
        self.get_signature(manifest)
            .map(TbSignature::new)
            .transpose()
            .context(EncodeSignatureSnafu { target })
    }

    #[tracing::instrument(skip(self))]
    pub fn build_tbpk(&mut self, target: Target) -> Result<(), PackWriterError> {
        info!("Building {target:?} pack");
        // the index comes from the chunks, and the manifest holds the index's hash
        self.chunk(target)?;

        // the signature covers the manifest's bytes, so the manifest has to exist first
        let manifest = self.get_signed_manifest(target)?;
        let signature = self.get_tb_signature(target, &manifest)?;
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
    #[tracing::instrument(skip(self), fields(path = %path.display()))]
    pub fn write_tbpk_out(&self, path: &Path) -> Result<(), PackWriterError> {
        let tbpk_value = self.tbpk.as_ref().context(NotBuiltSnafu { path })?;
        let toc = &tbpk_value.toc;
        let target = tbpk_value.manifest.parsed.body.target;
        let chunks = self.get_chunks(target)?;
        info!("Writing {target:?} pack to {}", path.display());

        let index =
            postcard::to_allocvec(&tbpk_value.index).context(EncodeIndexSnafu { target })?;
        ensure!(
            index.len() as u64 == toc.index_len,
            IndexLenMismatchSnafu {
                target,
                actual: index.len() as u64,
                expected: toc.index_len
            }
        );

        let signature = match tbpk_value.signature.as_ref() {
            None => Vec::new(),
            Some(sig) => sig.to_bytes().context(EncodeSignatureSnafu { target })?,
        };

        let file = File::create(path).context(CreateOutputSnafu { path })?;
        let mut out = BufWriter::new(file);
        for (section, bytes) in [
            ("header", &tbpk_value.header.to_bytes()[..]),
            ("table of contents", &toc.to_bytes()),
            ("manifest", &tbpk_value.manifest.raw),
            ("signature", &signature),
            ("index", &index),
        ] {
            out.write_all(bytes).context(WriteSectionSnafu {
                section,
                len: bytes.len(),
                path,
            })?;
            info!("Wrote {section} section, {} bytes", bytes.len());
        }

        let spool = chunks.data.path();
        let mut data = chunks.data.open().context(OpenSpoolSnafu { spool })?;
        let data_len = std::io::copy(&mut data, &mut out).context(CopyDataSnafu { spool, path })?;
        ensure!(
            data_len == toc.data_len,
            DataLenMismatchSnafu {
                target,
                actual: data_len,
                expected: toc.data_len
            }
        );
        info!("Wrote data section, {data_len} bytes");
        out.flush().context(FlushOutputSnafu { path })?;

        info!("Wrote tbpk to {}", path.display());
        Ok(())
    }
}

/// lowercase hex, for logging hashes and keys
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
