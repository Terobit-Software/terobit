use bevy::prelude::*;
use std::path::Path;
use tracing::info;

pub struct RuntimePackPlugin;

#[derive(Event)]
pub struct LoadPackFileEvent {
    pub path: Box<Path>,
}

impl RuntimePackPlugin {
    fn scan_pack(e: On<LoadPackFileEvent>) {
        let path = e.event().path.as_ref();

        info!(
            "Pack plugin recieved request to read pack at path: {:?}",
            path
        );
    }
}

impl Plugin for RuntimePackPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(Self::scan_pack);
    }
}
