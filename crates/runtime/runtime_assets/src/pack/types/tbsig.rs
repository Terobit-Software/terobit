use crate::pack::{Certificate, CommonHeader};
use serde_with::{Bytes, serde_as};

/// a `.tbsig` file
pub struct TbSignature {
    /// the common header, magic `TBSG`
    header: CommonHeader,
    /// the body, postcard encoded
    body: SignatureV1,
}

#[serde_as]
#[derive(serde::Serialize)]
pub struct SignatureV1 {
    /// the key that made `signature`. purpose must be Pack
    cert: Certificate,
    /// ed25519 by cert.public_key over hash(PACK, manifest bytes)
    #[serde_as(as = "Bytes")]
    signature: [u8; 64],
}

pub const TBSIG_CURRENT_VERSION: u16 = 1;
