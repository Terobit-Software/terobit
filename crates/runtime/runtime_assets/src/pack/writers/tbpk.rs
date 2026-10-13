use crate::pack::writers::{ManifestSource, SideAssets};
use crate::pack::{
    CommonHeader, FileEntry, HEADER_LEN, Handle, ManifestV1, TBMAN_CURRENT_VERSION,
    TBPK_CURRENT_VERSION, TableOfContents, TbManifest, TbPack,
};
use crate::pack::{DEFAULT_CHUNK_SIZE, MAX_CHUNK_SIZE, MIN_CHUNK_SIZE};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Default)]
pub struct PackWriter {}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_tbpk_out(&self) -> Result<(), String> {
        Ok(())
    }
}
