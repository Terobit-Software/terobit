use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{
    ChunkEntry, CommonHeader, FileEntry, FileLocation, Handle, IndexV1, MagicValue, ManifestV1,
    SignatureV1, Signed, TBPK_CURRENT_VERSION, TableOfContents, Target, TbManifest, TbPack,
    TbSignature,
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
        CommonHeader::new(MagicValue::TBPK, TBPK_CURRENT_VERSION, body_length)
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

    fn get_signed_manifest(&self, target: Target) -> Result<Signed<TbManifest>, String> {
        let manifest = match self.get_manifest(target) {
            None => return Err("no manifest exists, yet".into()),
            Some(v) => v,
        };
        TbManifest::encode(manifest)
    }

    fn get_signature(&self, target: Target) -> Option<SignatureV1> {}

    fn get_tb_signature(&self, target: Target) -> Result<Option<TbSignature>, String> {
        self.get_signature(target).map(TbSignature::new).transpose()
    }

    pub fn build_tbpk(&mut self, target: Target) -> Result<(), String> {
        let body_length = 0;

        let tbpk = TbPack {
            header: self.header(body_length, target),
            toc: self.get_table_of_contents(target)?,
            index: self.get_index(target)?,
            manifest: self.get_signed_manifest(target)?,
            signature: self.get_tb_signature(target)?,
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
