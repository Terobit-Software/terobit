/// the magic value of the file
pub enum MagicValue {
    /// terobit pack file
    TBPK,
    /// terobit manifest file
    TBMAN,
    /// terobit signature file
    TBSG,
    /// terobit license file
    TBLC,
    /// terobit resource file
    TBRS,
}

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
