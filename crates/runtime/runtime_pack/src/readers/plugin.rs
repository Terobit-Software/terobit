use crate::readers::PackReader;
use bevy::prelude::*;
use std::path::Path;
use tracing::info;

pub struct RuntimePackPlugin;

#[derive(Event)]
pub struct LoadPackFileEvent {
    pub path: Box<Path>,
}

#[derive(Resource, Default)]
pub struct RuntimePackConfig {
    allow_dir_read: bool,
}

impl RuntimePackConfig {
    pub fn get_allow_dir_read(&self) -> &bool {
        &self.allow_dir_read
    }
    pub fn set_allow_dir_read(&mut self, value: bool) {
        self.allow_dir_read = value;
        info!("RuntimePack allow_dir_read has been set to {}", value);
    }
}

impl RuntimePackPlugin {
    fn scan_pack(e: On<LoadPackFileEvent>, mut commands: Commands, config: Res<RuntimePackConfig>) {
        let path = e.event().path.as_ref();

        PackReader::read_pack(&mut commands, path, &config);
    }
}

impl Plugin for RuntimePackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Self::scan_pack);
        app.init_resource::<RuntimePackConfig>();
    }
}
