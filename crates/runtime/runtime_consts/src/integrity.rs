//! blake3 derive-key contexts, see docs/formats/common/INTEGRITY.md

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
