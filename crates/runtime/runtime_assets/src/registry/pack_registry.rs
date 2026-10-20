use crate::pack::AssetHandle;
use crate::registry::TbpkSource;
use bevy::ecs::resource::Resource;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Resource, Clone, Default)]
pub struct PackRegistry(Arc<RwLock<RegistryInner>>);

pub enum RootLocation {
    Developer { path: Box<Path> },
    Pack { id: u64 },
}

#[derive(Default)]
pub struct RegistryInner {
    /// <RealmId, TbpkSource>
    sources: HashMap<u64, TbpkSource>,
}
