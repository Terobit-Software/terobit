use crate::pack::{Certificate, CommonHeader};

/// a `.tbsig` file
pub struct TbSignature {
    /// the common header, magic `TBSG`
    header: CommonHeader,
    /// the body, postcard encoded
    body: SignatureV1,
}

pub struct SignatureV1 {
    /// the key that made `signature`. purpose must be Pack
    cert: Certificate,
    /// ed25519 by cert.public_key over hash(PACK, manifest bytes)
    signature: [u8; 64],
}

pub const TBSIG_CURRENT_VERSION: u16 = 1;
