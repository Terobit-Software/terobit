/// the magic value of the file
pub enum MagicValue {
    /// terobit pack file
    TBPK,
    /// terobit manifest file
    TBMN,
    /// terobit signature file
    TBSG,
    /// terobit license file
    TBLC,
    /// terobit resource file
    TBRS,
}

/// size of the common header in bytes. fixed width, so it never depends on the struct layout
pub const HEADER_LEN: u32 = 24;

/// not serde, always encode with `to_bytes` so it stays fixed width
pub struct CommonHeader {
    /// the magic value of the file
    pub magic: MagicValue,
    /// the version of the file type, starts at 1
    pub version: u16,
    /// reserved for file type
    pub flags: u16,
    /// length of the header from the start of the file in bytes
    pub header_length: u32,
    /// reserved for file type
    pub reserved: u32,
    /// in bytes the length of the body after the header (file length is `header_length + body_length`)
    pub body_length: u64,
}

impl MagicValue {
    /// the 4 ascii bytes at the start of the file
    pub fn bytes(&self) -> [u8; 4] {
        match self {
            MagicValue::TBPK => *b"TBPK",
            MagicValue::TBMN => *b"TBMN",
            MagicValue::TBSG => *b"TBSG",
            MagicValue::TBLC => *b"TBLC",
            MagicValue::TBRS => *b"TBRS",
        }
    }
}

impl CommonHeader {
    pub fn new(magic: MagicValue, version: u16, body_length: u64) -> Self {
        Self {
            magic,
            version,
            flags: 0,
            header_length: HEADER_LEN,
            reserved: 0,
            body_length,
        }
    }

    /// fixed width little endian. not postcard, which would varint the integers
    pub fn to_bytes(&self) -> [u8; HEADER_LEN as usize] {
        let mut out = [0u8; HEADER_LEN as usize];
        out[0..4].copy_from_slice(&self.magic.bytes());
        out[4..6].copy_from_slice(&self.version.to_le_bytes());
        out[6..8].copy_from_slice(&self.flags.to_le_bytes());
        out[8..12].copy_from_slice(&self.header_length.to_le_bytes());
        out[12..16].copy_from_slice(&self.reserved.to_le_bytes());
        out[16..24].copy_from_slice(&self.body_length.to_le_bytes());
        out
    }
}

/// encodes a whole file, the common header then the postcard encoded body
pub fn encode_file<T: serde::Serialize>(
    magic: MagicValue,
    version: u16,
    body: &T,
) -> Result<(CommonHeader, Vec<u8>), String> {
    let body_bytes = postcard::to_allocvec(body).map_err(|e| e.to_string())?;
    let header = CommonHeader::new(magic, version, body_bytes.len() as u64);

    let mut out = Vec::with_capacity(HEADER_LEN as usize + body_bytes.len());
    out.extend_from_slice(&header.to_bytes());
    out.extend_from_slice(&body_bytes);
    Ok((header, out))
}
