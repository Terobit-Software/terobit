/// the magic value of the file
enum MagicValue {
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

pub struct CommonHeader {
    /// the magic value of the file
    magic: MagicValue,
    /// the version of the file type, starts at 1
    version: u16,
    /// reserved for file type
    flags: u16,
    /// length of the header from the start of the file in bytes
    header_length: u32,
    /// reserved for file type
    reserved: u32,
    /// in bytes the length of the body after the header (file length is `header_length + body_length`)
    body_length: u32,
}
