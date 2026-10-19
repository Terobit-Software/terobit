use bevy::{ecs::error::info, prelude::*};
use std::fs::File;
use std::path::Path;
use tracing::{error, info};

use crate::pack::{
    CommonHeader, HEADER_LEN,
    readers::{ManifestSource, RuntimePackConfig},
};

pub struct PackReader {}

impl PackReader {
    pub fn read_pack(
        commands: &mut Commands,
        path: &Path,
        config: &RuntimePackConfig,
    ) -> Result<(), String> {
        info!("Reading pack at path {:?}", path);

        if path.is_dir() {
            if !*config.get_allow_dir_read() {
                return Err("Path is directory, but allow_dir_read is NOT enabled. Failed to read pack file".to_string());
            }

            info!("Path is directory, continuing as a developer package. Assuming newest version");

            let manifest_source = ManifestSource::from_path(&path.join("manifest.toml"))?;

            return Ok(());
        }

        info!("path is not directory, assuming is of type file");

        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                return Err(format!("failed to open file: {}", e));
            }
        };

        let metadata = match file.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                return Err(format!("failed to retrieve metadata: {}", e));
            }
        };

        let len = metadata.len();
        if len < HEADER_LEN as u64 {
            return Err(format!("file is invalid, too short, len {} bytes", len));
        } else {
            info!(len);
        }

        Ok(())
    }
}
