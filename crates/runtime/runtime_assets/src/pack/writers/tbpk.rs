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
pub struct PackWriter {
    tbpk: Option<TbPack>,
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

    pub fn build_tbpk(&mut self) -> Result<(), String> {
        let body_length = 0;

        let tbpk = TbPack {
            header: self.header(body_length),
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
