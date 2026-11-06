use crate::units::{KIB, MIB};

/// size of the common header in bytes. fixed width, so it never depends on the struct layout
pub const HEADER_LEN: u32 = 24;

pub const TBPK_CURRENT_VERSION: u16 = 1;
pub const TBMAN_CURRENT_VERSION: u16 = 1;
pub const TBSIG_CURRENT_VERSION: u16 = 1;
pub const TBLIC_CURRENT_VERSION: u16 = 1;

/// size of the table of contents in bytes, right after the common header
pub const TOC_LEN: u64 = 64;
/// most bytes the manifest section can take
pub const MAX_MANIFEST_LEN: u64 = 64 * MIB;
/// most bytes the signature section can take, 0 means unsigned
pub const MAX_SIGNATURE_LEN: u64 = 4 * KIB;
/// smallest chunk size a reader accepts
pub const MIN_CHUNK_SIZE: u32 = 64 * KIB as u32;
/// largest chunk size a reader accepts
pub const MAX_CHUNK_SIZE: u32 = 16 * MIB as u32;
/// chunk size the v1 pack tool writes
pub const DEFAULT_CHUNK_SIZE: u32 = MIB as u32;
/// files this size or bigger start their own chunks when writing
pub const LARGE_FILE_THRESHOLD: u64 = 256 * KIB;

/// zstd level used to compress chunks in packs whose manifest is v1
pub const ZSTD_LEVEL: i32 = 5;
/// a chunk is stored compressed only if zstd saves at least this percent of its raw bytes
pub const MIN_COMPRESSION_SAVINGS_PERCENT: u64 = 20;

/// most bytes a whole pack path can take
pub const MAX_PATH_LEN: usize = 255;
/// most segments a pack path can have
pub const MAX_PATH_SEGMENTS: usize = 16;
/// most bytes one pack path segment can take
pub const MAX_SEGMENT_LEN: usize = 64;
/// names windows reserves, with or without an extension
pub const RESERVED_NAMES: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// the only handle namespace supported for now
pub const SELF_NAMESPACE: &str = "self";
/// the manifest a dev directory is read from, at its root
pub const DEV_MANIFEST_FILE: &str = "manifest.toml";
/// extension every script file must have
pub const SCRIPT_EXTENSION: &str = ".wasm";
/// name of the bevy asset source packs are served from, `pack://<pack path>`
pub const PACK_ASSET_SOURCE: &str = "pack";
