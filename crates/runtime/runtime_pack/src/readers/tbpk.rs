use bevy::prelude::*;
use std::fs::File;
use std::path::Path;
use tracing::info;

pub struct PackReader {}

impl PackReader {
    pub fn read_pack(commands: &mut Commands, path: &Path) {
        info!("Reading pack at path {:?}", path);

        if path.is_dir() {
            info!("Path is directory, continuing as a developer package.")
        }

        let file = File::open(path).expect("Pack file not found");
    }
}
