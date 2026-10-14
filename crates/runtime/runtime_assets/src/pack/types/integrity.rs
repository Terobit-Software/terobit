use serde_with::{Bytes, serde_as};

/// blake3 derive-key context for a chunk's stored bytes
pub const CHUNK: &str = "terobit 2026-10-01 chunk";
/// blake3 derive-key context for a pack index's bytes
pub const INDEX: &str = "terobit 2026-10-01 pack index";
/// blake3 derive-key context for a public key, to make a key id
pub const KEY_ID: &str = "terobit 2026-10-01 key id";
/// blake3 derive-key context for what a root key signs
pub const CERT: &str = "terobit 2026-10-01 certificate";
/// blake3 derive-key context for what a pack signing key signs
pub const PACK: &str = "terobit 2026-10-01 pack signature";
/// blake3 derive-key context for what a license signing key signs
pub const LICENSE: &str = "terobit 2026-10-01 license signature";

#[serde_as]
#[derive(serde::Serialize)]
pub struct Certificate {
    /// what this key may sign
    purpose: Purpose,
    /// the signing key's ed25519 public key
    #[serde_as(as = "Bytes")]
    public_key: [u8; 32],
    /// first 8 bytes of hash(KEY_ID, root public key). picks the root key, not a security check
    #[serde_as(as = "Bytes")]
    root_key_id: [u8; 8],
    /// root key's ed25519 signature over hash(CERT, [purpose as u8] ++ public_key)
    #[serde_as(as = "Bytes")]
    root_signature: [u8; 64],
}

#[derive(serde::Serialize)]
pub enum Purpose {
    Pack = 0,
    License = 1,
}
