use crate::{Certificate, CommonHeader};

/// a `.tblic` file
pub struct TbLicense {
    /// the common header, magic `TBLC`
    header: CommonHeader,
    /// the body, postcard encoded
    body: LicenseV1,
}

pub struct LicenseV1 {
    /// the user who owns the realm
    user_id: u64,
    /// the realm they own
    realm_id: u64,
    /// the key that made `signature`. purpose must be License
    cert: Certificate,
    /// ed25519 by cert.public_key over hash(LICENSE, terms)
    signature: [u8; 64],
}

pub const CURRENT_VERSION: u16 = 1;
