use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{
    ChunkEntry, CommonHeader, FileEntry, FileLocation, HEADER_LEN, Handle, IndexV1, ManifestV1,
    SignatureV1, TBMAN_CURRENT_VERSION, TBPK_CURRENT_VERSION, TableOfContents, Target, TbManifest,
    TbPack,
};
use crate::pack::{DEFAULT_CHUNK_SIZE, MAX_CHUNK_SIZE, MIN_CHUNK_SIZE};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct PackWriter {
    // implemented
    tbpk: Option<TbPack>,
    // implemented
    chunk_size: u32,
    // needs implementation
    chunk_entries: Vec<ChunkEntry>,
    // needs implementation
    file_locations: Vec<FileLocation>,
}

impl Default for PackWriter {
    fn default() -> Self {
        Self {
            tbpk: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunk_entries: Vec::new(),
            file_locations: Vec::new(),
        }
    }
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    fn header(&self, body_length: u64, target: Target) -> CommonHeader {
        CommonHeader {
            magic: crate::pack::MagicValue::TBMN,
            version: TBPK_CURRENT_VERSION,
            flags: 0,
            header_length: HEADER_LEN,
            reserved: 0,
            body_length,
        }
    }

    fn get_chunk_entries(&self, target: Target) -> &Vec<ChunkEntry> {
        &self.chunk_entries
    }

    fn get_file_locations(&self, target: Target) -> &Vec<FileLocation> {
        &self.file_locations
    }

    fn get_index(&self, target: Target) -> Result<IndexV1, String> {
        let index = IndexV1 {
            chunk_size: self.chunk_size,
            chunks: self.get_chunk_entries(target).clone(),
            files: self.get_file_locations(target).clone(),
        };

        Ok(index)
    }

    fn get_table_of_contents(&self, target: Target) -> Result<TableOfContents, String> {}

    fn get_manifest(&self, target: Target) -> Option<ManifestV1> {}

    fn get_manifest_u8_vec(&self, target: Target) -> Result<Vec<u8>, String> {
        let manifest = match self.get_manifest(target) {
            None => return Err("no manifest exists, yet".into()),
            Some(v) => v,
        };
        postcard::to_allocvec::<ManifestV1>(&manifest).map_err(|e| e.to_string())
    }

    fn get_signature(&self, target: Target) -> Option<SignatureV1> {}

    // i fear for future me :sob:
    // typeception bro
    fn get_signature_u8_vec(&self, target: Target) -> Result<Option<Vec<u8>>, String> {
        let signature = match self.get_signature(target) {
            None => return Ok(None),
            Some(v) => v,
        };
        let v = match postcard::to_allocvec::<SignatureV1>(&signature) {
            Err(e) => return Err(e.to_string()),
            Ok(v) => v,
        };
        Ok(Some(v))
    }

    pub fn build_tbpk(&mut self, target: Target) -> Result<(), String> {
        let body_length = 0;

        let tbpk = TbPack {
            header: self.header(body_length, target),
            toc: self.get_table_of_contents(target)?,
            index: self.get_index(target)?,
            manifest: self.get_manifest_u8_vec(target)?,
            signature: self.get_signature_u8_vec(target)?,
        };

        self.tbpk = Some(tbpk);

        Ok(())
    }

    pub fn write_tbpk_out(&self) -> Result<(), String> {
        let tbpk_value = match self.tbpk.as_ref() {
            None => return Err("TBPK must be built before calling write out".into()),
            Some(v) => v,
        };

        Ok(())
    }
}
