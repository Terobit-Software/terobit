use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{
    ChunkEntry, CommonHeader, FileEntry, FileLocation, HEADER_LEN, Handle, IndexV1, ManifestV1,
    TBMAN_CURRENT_VERSION, TBPK_CURRENT_VERSION, TableOfContents, TbManifest, TbPack,
};
use crate::pack::{DEFAULT_CHUNK_SIZE, MAX_CHUNK_SIZE, MIN_CHUNK_SIZE};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct PackWriter {
    tbpk: Option<TbPack>,
    chunk_size: u32,
    chunk_entries: Vec<ChunkEntry>,
    file_entries: Vec<FileLocation>,
}

impl Default for PackWriter {
    fn default() -> Self {
        Self {
            tbpk: None,
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunk_entries: Vec::new(),
            file_entries: Vec::new(),
        }
    }
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    fn header(&self, body_length: u64) -> CommonHeader {
        CommonHeader {
            magic: crate::pack::MagicValue::TBMN,
            version: TBPK_CURRENT_VERSION,
            flags: 0,
            header_length: HEADER_LEN,
            reserved: 0,
            body_length,
        }
    }

    fn get_chunk_entries(&self) -> &Vec<ChunkEntry> {
        &self.chunk_entries
    }

    fn get_file_locations(&self) -> &Vec<FileLocation> {
        &self.file_entries
    }

    fn get_index(&self) -> Result<IndexV1, String> {
        let index = IndexV1 {
            chunk_size: self.chunk_size,
            chunks: self.get_chunk_entries().clone(),
            files: self.get_file_locations().clone(),
        };

        Ok(index)
    }

    fn get_table_of_contents(&self) -> Result<TableOfContents, String> {}

    fn get_manifest_u8_vec(&self) -> Result<Vec<u8>, String> {}

    fn get_signature_u8_vec(&self) -> Option<Vec<u8>> {}

    pub fn build_tbpk(&mut self) -> Result<(), String> {
        let body_length = 0;

        let tbpk = TbPack {
            header: self.header(body_length),
            toc: self.get_table_of_contents()?,
            index: self.get_index()?,
            manifest: self.get_manifest_u8_vec()?,
            signature: self.get_signature_u8_vec(),
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
