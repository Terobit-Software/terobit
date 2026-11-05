use bevy::ecs::{event::Event, observer::On};
use runtime_assets::pack::AssetHandle;

#[derive(Event)]
pub struct LoadModuleEvent {
    pub handle: AssetHandle,
}

pub fn observe_load_module_event(event: On<LoadModuleEvent>) {
    let handle = event.handle.clone();
    tracing::info!("New load module request from {}", handle);
}
