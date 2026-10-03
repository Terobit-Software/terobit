use bevy::prelude::*;
use std::path::Path;

use crate::PackReader;

pub struct RuntimePackPlugin;

#[derive(Event)]
pub struct LoadPackFileEvent {
    pub path: Box<Path>,
}

#[derive(Resource, Default)]
pub struct RuntimePackConfig {
    allow_dirs: bool,
}

impl RuntimePackConfig {
    pub fn get_allow_dir_read(&self) -> &bool {
        &self.allow_dirs
    }
    pub fn set_allow_dir_read(&mut self, value: bool) {
        self.allow_dirs = value;
    }
}

impl RuntimePackPlugin {
    fn scan_pack(e: On<LoadPackFileEvent>, mut commands: Commands) {
        let path = e.event().path.as_ref();

        PackReader::read_pack(&mut commands, path);
    }
}

impl Plugin for RuntimePackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Self::scan_pack);
        app.init_resource::<RuntimePackConfig>();
    }
}
