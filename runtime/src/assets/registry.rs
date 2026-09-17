use std::collections::HashMap;
use std::path::Path;

use crate::assets::asset_error::AssetError;
use crate::assets::mesh_asset::{MeshAsset, MeshData, Vertex};
use crate::assets::raw_asset::{BUILTIN_CUBE_PATH, RawAsset, RawAssetData};
use crate::assets::shader_asset::{ShaderAsset, ShaderData};

pub(crate) type AssetId = usize;

/// Owns every asset's actual data. [`RawAsset`], [`MeshAsset`], and
/// [`ShaderAsset`] are just [`AssetId`] handles into the maps here — they
/// hold no data of their own, so all reads and future writes go through the
/// registry.
pub(crate) struct AssetRegistry {
    raw_assets: HashMap<AssetId, RawAssetData>,
    raw_ids: HashMap<Box<Path>, AssetId>,
    next_raw_id: AssetId,

    meshes: HashMap<AssetId, MeshData>,
    mesh_ids: HashMap<Box<Path>, AssetId>,
    next_mesh_id: AssetId,

    shaders: HashMap<AssetId, ShaderData>,
    shader_ids: HashMap<Box<Path>, AssetId>,
    next_shader_id: AssetId,
}

impl AssetRegistry {
    pub(crate) fn new() -> Result<Self, AssetError> {
        let mut reg = Self {
            raw_assets: HashMap::new(),
            raw_ids: HashMap::new(),
            next_raw_id: 0,

            meshes: HashMap::new(),
            mesh_ids: HashMap::new(),
            next_mesh_id: 0,

            shaders: HashMap::new(),
            shader_ids: HashMap::new(),
            next_shader_id: 0,
        };

        // loaded first so the built-in cube always gets asset id 0
        reg.read_mesh_from_path(Path::new(BUILTIN_CUBE_PATH))?;

        Ok(reg)
    }

    fn read_raw_from_path(&mut self, path: &Path) -> Result<RawAsset, AssetError> {
        if let Some(&id) = self.raw_ids.get(path) {
            return Ok(RawAsset::new(id));
        }

        let data = RawAssetData::read(path)?;

        let id = self.next_raw_id;
        self.next_raw_id += 1;

        self.raw_ids
            .insert(path.to_path_buf().into_boxed_path(), id);
        self.raw_assets.insert(id, data);

        Ok(RawAsset::new(id))
    }

    /// Loads (and caches) the mesh at `path`, returning a stable [`AssetId`]
    /// that [`Self::get_mesh`] can later resolve back to the parsed mesh
    /// geometry. Repeated calls with the same path return the same id
    /// without re-parsing.
    pub(crate) fn read_mesh_from_path(&mut self, path: &Path) -> Result<AssetId, AssetError> {
        if let Some(&id) = self.mesh_ids.get(path) {
            return Ok(id);
        }

        let raw = self.read_raw_from_path(path)?;
        let mesh = MeshData::from_raw(self.raw_assets.get(&raw.id()).unwrap())?;

        let id = self.next_mesh_id;
        self.next_mesh_id += 1;

        self.mesh_ids
            .insert(path.to_path_buf().into_boxed_path(), id);
        self.meshes.insert(id, mesh);

        Ok(id)
    }

    /// Loads (and caches) the shader source at `path`, returning a
    /// [`ShaderAsset`] handle that resolves back to the source through the
    /// registry. Repeated calls with the same path return a handle to the
    /// same cached entry.
    #[allow(dead_code)]
    pub(crate) fn read_shader_from_path(&mut self, path: &Path) -> Result<ShaderAsset, AssetError> {
        if let Some(&id) = self.shader_ids.get(path) {
            return Ok(ShaderAsset::new(id));
        }

        let raw = self.read_raw_from_path(path)?;
        let shader = ShaderData::from_raw(self.raw_assets.get(&raw.id()).unwrap());

        let id = self.next_shader_id;
        self.next_shader_id += 1;

        self.shader_ids
            .insert(path.to_path_buf().into_boxed_path(), id);
        self.shaders.insert(id, shader);

        Ok(ShaderAsset::new(id))
    }

    #[inline]
    pub(crate) fn get_mesh(&self, id: AssetId) -> Option<MeshAsset> {
        self.meshes.contains_key(&id).then(|| MeshAsset::new(id))
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn get_shader(&self, id: AssetId) -> Option<ShaderAsset> {
        self.shaders.contains_key(&id).then(|| ShaderAsset::new(id))
    }

    // --- low-level data accessors backing the handle structs' getters ---

    #[inline]
    pub(crate) fn raw_data(&self, id: AssetId) -> &[u8] {
        self.raw_assets
            .get(&id)
            .expect("RawAsset handle outlived its registry entry")
            .data()
    }

    #[inline]
    pub(crate) fn raw_is_builtin(&self, id: AssetId) -> bool {
        self.raw_assets
            .get(&id)
            .expect("RawAsset handle outlived its registry entry")
            .is_builtin()
    }

    #[inline]
    pub(crate) fn raw_path(&self, id: AssetId) -> &Path {
        self.raw_assets
            .get(&id)
            .expect("RawAsset handle outlived its registry entry")
            .path()
    }

    #[inline]
    pub(crate) fn mesh_verts(&self, id: AssetId) -> &[Vertex] {
        self.meshes
            .get(&id)
            .expect("MeshAsset handle outlived its registry entry")
            .verts()
    }

    #[inline]
    pub(crate) fn mesh_indices(&self, id: AssetId) -> &[u16] {
        self.meshes
            .get(&id)
            .expect("MeshAsset handle outlived its registry entry")
            .indices()
    }

    #[inline]
    pub(crate) fn shader_data(&self, id: AssetId) -> &str {
        self.shaders
            .get(&id)
            .expect("ShaderAsset handle outlived its registry entry")
            .data()
    }

    #[inline]
    pub(crate) fn shader_path(&self, id: AssetId) -> &Path {
        self.shaders
            .get(&id)
            .expect("ShaderAsset handle outlived its registry entry")
            .path()
    }
}
