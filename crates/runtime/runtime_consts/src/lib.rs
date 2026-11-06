//! Constants shared across the runtime, its binaries and scripts. No dependencies, so any
//! crate can use it without pulling anything else in.
//!
//! Format constants (`pack`, `integrity`) are part of the on-disk formats in `docs/formats`,
//! changing one is a format version bump.

#![no_std]

pub mod units {
    pub const KIB: u64 = 1024;
    pub const MIB: u64 = 1024 * KIB;
}

pub mod pack {
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
}

/// blake3 derive-key contexts, see docs/formats/common/INTEGRITY.md
pub mod integrity {
    /// context for a chunk's stored bytes
    pub const CHUNK: &str = "terobit 2026-10-01 chunk";
    /// context for a pack index's bytes
    pub const INDEX: &str = "terobit 2026-10-01 pack index";
    /// context for a public key, to make a key id
    pub const KEY_ID: &str = "terobit 2026-10-01 key id";
    /// context for what a root key signs
    pub const CERT: &str = "terobit 2026-10-01 certificate";
    /// context for what a pack signing key signs
    pub const PACK: &str = "terobit 2026-10-01 pack signature";
    /// context for what a license signing key signs
    pub const LICENSE: &str = "terobit 2026-10-01 license signature";
}

/// run flags read through `RuntimeFlags`
pub mod flags {
    /// turns on developer mode, needs the client's `dev` feature
    pub const DEV: &str = "--dev";
    /// the pack or dev directory developer mode loads
    pub const PACK_FILE: &str = "--pack-file";
}

pub mod sandbox {
    /// fuel each wasm store starts with
    pub const WASM_FUEL: u64 = 1_000_000;
}

/// `(first, last)` names passed to `init_logging`, see its docs for the casing
pub mod logging {
    pub const CLIENT: (&str, &str) = ("Terobit", "Client");
    pub const TBPK_GENERATOR: (&str, &str) = ("Tbpk", "Generator");
    pub const RUN_CLIENT_SCRIPT: (&str, &str) = ("Tbrun", "Client");
    pub const EXAMPLE_GENERATOR: (&str, &str) = ("Tbpk", "Regen");
}

/// folders and names used by the binaries and scripts, relative to the workspace root
pub mod paths {
    /// where `tbpk_generator` writes packs when `--out` isn't given
    pub const TBPK_GENERATOR_OUT: &str = "out/tbpk_generator";
    /// where `run_client_script` stages the client and its assets
    pub const CLIENT_OUT: &str = "out/client";
    /// the client binary's package and file name
    pub const CLIENT_BIN: &str = "terobit_client";
}
