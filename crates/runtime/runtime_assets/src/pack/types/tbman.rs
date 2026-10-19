use crate::pack::{CommonHeader, EncodeError, MagicValue, Signed, encode_file};

/// zstd level used to compress chunks in packs whose manifest is v1
pub const ZSTD_LEVEL: i32 = 5;

/// a `.tbman` file. not serde, encode with `TbManifest::encode`
pub struct TbManifest {
    /// the common header, magic `TBMN`
    pub header: CommonHeader,
    /// the body, postcard encoded
    pub body: ManifestV1,
}

impl TbManifest {
    /// encodes the whole file once, so the bytes that get signed are the bytes that get written
    pub fn encode(body: ManifestV1) -> Result<Signed<TbManifest>, EncodeError> {
        let (header, raw) = encode_file(MagicValue::TBMN, TBMAN_CURRENT_VERSION, &body)?;
        Ok(Signed {
            raw,
            parsed: TbManifest { header, body },
        })
    }
}

#[derive(serde::Serialize)]
pub struct ManifestV1 {
    /// which side this pack is for. covered by the signature, so it can't be swapped
    pub target: Target,
    /// the realm (game) this pack belongs to
    pub realm_id: u64,
    /// the developer who owns the realm
    pub owner_id: u64,
    /// whether playing needs a license. see tblic
    pub access: Access,
    /// version of this pack's content, bumped on every publish. the client and server
    /// pack from one publish share the same value, and online play rejects a client whose
    /// pack_version doesn't match the server's.
    /// not related to the format version in the header.
    pub pack_version: u32,
    /// sorted by path, byte order
    pub files: Vec<FileEntry>,
    pub assets: Vec<AssetEntry>,
    pub scripts: Vec<ScriptEntry>,
    /// the pack index, which says where every file's bytes are and holds every chunk's hash
    pub index: IndexRef,
}

#[derive(serde::Serialize, Copy, Clone, Debug)]
pub enum Target {
    Client = 0,
    Server = 1,
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Access {
    /// anyone can play
    Free = 0,
    /// the client needs a valid license for this realm and the last logged in user
    Licensed = 1,
}

#[derive(serde::Serialize)]
pub struct FileEntry {
    /// see path rules in conventions
    pub path: String,
    /// uncompressed size in bytes
    pub size: u64,
}

#[derive(serde::Serialize)]
pub struct AssetEntry {
    /// `namespace:name`, see conventions
    pub id: String,
    /// index into `files`
    pub file: u32,
}

#[derive(serde::Serialize)]
pub struct ScriptEntry {
    /// `namespace:name`, see conventions
    pub id: String,
    /// index into `files`, must be a `.wasm` file
    pub file: u32,
}

#[derive(serde::Serialize)]
pub struct IndexRef {
    /// exact length of the index in bytes
    pub len: u64,
    /// hash(INDEX, index bytes), see integrity
    pub hash: [u8; 32],
}

pub const TBMAN_CURRENT_VERSION: u16 = 1;
