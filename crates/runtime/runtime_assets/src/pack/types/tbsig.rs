use crate::pack::{Certificate, CommonHeader, MagicValue, encode_file};
use serde_with::{Bytes, serde_as};

/// a `.tbsig` file
pub struct TbSignature {
    /// the common header, magic `TBSG`
    header: CommonHeader,
    /// the body, postcard encoded
    body: SignatureV1,
}

impl TbSignature {
    pub fn new(body: SignatureV1) -> Result<Self, String> {
        let (header, _) = encode_file(MagicValue::TBSG, TBSIG_CURRENT_VERSION, &body)?;
        Ok(Self { header, body })
    }

    /// the whole `.tbsig` file, header then body
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        encode_file(MagicValue::TBSG, TBSIG_CURRENT_VERSION, &self.body).map(|(_, raw)| raw)
    }

    pub fn header(&self) -> &CommonHeader {
        &self.header
    }

    pub fn body(&self) -> &SignatureV1 {
        &self.body
    }
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

impl SignatureV1 {
    pub fn new(cert: Certificate, signature: [u8; 64]) -> Self {
        Self { cert, signature }
    }

    pub fn cert(&self) -> &Certificate {
        &self.cert
    }

    pub fn signature(&self) -> &[u8; 64] {
        &self.signature
    }
}

pub const TBSIG_CURRENT_VERSION: u16 = 1;
