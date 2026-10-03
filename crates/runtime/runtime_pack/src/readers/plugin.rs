use bevy::prelude::*;
use std::path::Path;

use crate::PackReader;

pub struct RuntimePackPlugin;

#[derive(Event)]
pub struct LoadPackFileEvent {
    pub path: Box<Path>,
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
    }
}
