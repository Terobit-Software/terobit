use crate::CommonHeader;

/// zstd level used to compress chunks in packs whose manifest is v1
pub const ZSTD_LEVEL: i32 = 19;

/// a `.tbman` file
pub struct TbManifest {
    /// the common header, magic `TBMN`
    header: CommonHeader,
    /// the body, postcard encoded
    body: ManifestV1,
}

pub struct ManifestV1 {
    /// which side this pack is for. covered by the signature, so it can't be swapped
    target: Target,
    /// the realm (game) this pack belongs to
    realm_id: u64,
    /// the developer who owns the realm
    owner_id: u64,
    /// whether playing needs a license. see tblic
    access: Access,
    /// version of this pack's content, bumped on every publish. the client and server
    /// pack from one publish share the same value, and online play rejects a client whose
    /// pack_version doesn't match the server's.
    /// not related to the format version in the header.
    pack_version: u32,
    /// sorted by path, byte order
    files: Vec<FileEntry>,
    assets: Vec<AssetEntry>,
    scripts: Vec<ScriptEntry>,
    /// the pack index, which says where every file's bytes are and holds every chunk's hash
    index: IndexRef,
}

pub enum Target {
    Client = 0,
    Server = 1,
}

pub enum Access {
    /// anyone can play
    Free = 0,
    /// the client needs a valid license for this realm and the last logged in user
    Licensed = 1,
}

pub struct FileEntry {
    /// see path rules in conventions
    path: String,
    /// uncompressed size in bytes
    size: u64,
}

pub struct AssetEntry {
    /// `namespace:name`, see conventions
    id: String,
    /// index into `files`
    file: u32,
}

pub struct ScriptEntry {
    /// `namespace:name`, see conventions
    id: String,
    /// index into `files`, must be a `.wasm` file
    file: u32,
}

pub struct IndexRef {
    /// exact length of the index in bytes
    len: u64,
    /// hash(INDEX, index bytes), see integrity
    hash: [u8; 32],
}

pub const CURRENT_VERSION: u16 = 1;
