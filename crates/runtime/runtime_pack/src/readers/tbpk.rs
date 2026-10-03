use bevy::prelude::*;
use std::fs::File;
use std::path::Path;
use tracing::{error, info};

use crate::CommonHeader;

pub struct PackReader {}

impl PackReader {
    pub fn read_pack(commands: &mut Commands, path: &Path) {
        info!("Reading pack at path {:?}", path);

        if path.is_dir() {
            info!("Path is directory, continuing as a developer package. Assuming newest version")
        }

        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                error!("failed to open file: {}", e);
                return;
            }
        };

        let metadata = match file.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                error!("failed to retrieve metadata: {}", e);
                return;
            }
        };

        let len = metadata.len();
        if len < size_of::<CommonHeader>() as u64 {
            error!("file is invalid, too short, len {} bytes", len);
        } else {
            info!(len);
        }
    }
}
