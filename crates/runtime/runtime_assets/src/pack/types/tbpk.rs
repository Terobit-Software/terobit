use crate::pack::CommonHeader;

/// size of the table of contents in bytes, right after the common header
pub const TOC_LEN: u64 = 64;
/// most bytes the manifest section can take
pub const MAX_MANIFEST_LEN: u64 = 64 * 1024 * 1024;
/// most bytes the signature section can take, 0 means unsigned
pub const MAX_SIGNATURE_LEN: u64 = 4 * 1024;
/// smallest chunk size a reader accepts
pub const MIN_CHUNK_SIZE: u32 = 64 * 1024;
/// largest chunk size a reader accepts
pub const MAX_CHUNK_SIZE: u32 = 16 * 1024 * 1024;
/// chunk size the v1 pack tool writes
pub const DEFAULT_CHUNK_SIZE: u32 = 1024 * 1024;
/// files this size or bigger start their own chunks when writing
pub const LARGE_FILE_THRESHOLD: u64 = 256 * 1024;

/// a `.tbpk` archive, everything except the data section, which is read chunk by chunk
pub struct TbPack {
    /// the common header, magic `TBPK`
    pub header: CommonHeader,
    /// where each section is in the file
    pub toc: TableOfContents,
    /// the manifest section, a complete `.tbman` file
    pub manifest: Vec<u8>,
    /// the signature section, a complete `.tbsig` file, `None` if unsigned
    pub signature: Option<Vec<u8>>,
    /// the index section
    pub index: IndexV1,
}

/// fixed width little endian, right after the common header. offsets are from the start of the file
pub struct TableOfContents {
    pub manifest_offset: u64,
    pub manifest_len: u64,
    pub signature_offset: u64,
    pub signature_len: u64,
    pub index_offset: u64,
    pub index_len: u64,
    pub data_offset: u64,
    pub data_len: u64,
}

/// the index section, postcard encoded. its length and hash are in the signed manifest
pub struct IndexV1 {
    /// most raw bytes a chunk can hold. 1 mib when written by the v1 tool, readers accept
    /// any power of two from 64 kib to 16 mib
    pub chunk_size: u32,
    /// in data section order
    pub chunks: Vec<ChunkEntry>,
    /// same length and order as the manifest's `files`
    pub files: Vec<FileLocation>,
}

pub struct ChunkEntry {
    /// where the stored bytes start, from the start of the data section
    pub offset: u64,
    /// bytes in the data section
    pub stored_len: u32,
    /// bytes after decompression, 1 to chunk_size
    pub raw_len: u32,
    pub compression: Compression,
    /// hash(CHUNK, stored bytes), see integrity
    pub hash: [u8; 32],
}

pub enum Compression {
    /// stored bytes are the raw bytes. stored_len equals raw_len
    None = 0,
    /// zstd, version and level pinned by the manifest version
    Zstd = 1,
}

pub struct FileLocation {
    /// the chunk the file starts in
    pub first_chunk: u32,
    /// where the file starts in that chunk's raw bytes
    pub offset_in_chunk: u32,
}

pub const TBPK_CURRENT_VERSION: u16 = 1;
