use crate::assets::raw_asset::RawAssetData;
use crate::assets::registry::{AssetId, AssetRegistry};
use std::path::{Path, PathBuf};
use tracing::warn;

/// Parsed shader source. Owned exclusively by the [`AssetRegistry`]; nothing
/// outside the registry holds this data directly.
pub(crate) struct ShaderData {
    data: String,
    // used box path for less memory footprint
    path: Box<Path>,
    // it is reccomended to use a fixed length vec
    shader_type: ShaderType,
}

pub(crate) enum ShaderType {
    Vertex,
    Fragment,
    Compute,
    VertexFragment,
}

impl ShaderData {
    pub(crate) fn from_raw(raw: &RawAssetData, shader_type: ShaderType) -> Self {
        if raw.is_builtin() {
            match raw.data() {
                b"bi.rootshader" => {
                    return Self {
                        data: String::from(include_str!("shaders/root.wgsl")),
                        path: raw.path().to_owned().into_boxed_path(),
                        shader_type: ShaderType::VertexFragment,
                    };
                }
                b"bi.frustumshader" => {
                    return Self {
                        data: String::from(include_str!("shaders/frustum.wgsl")),
                        path: raw.path().to_owned().into_boxed_path(),
                        shader_type: ShaderType::Compute,
                    };
                }
                _ => {}
            }
        }

        Self {
            data: String::from_utf8_lossy(raw.data()).into_owned(),
            path: raw.path().to_owned().into_boxed_path(),
            shader_type,
        }
    }

    #[inline]
    pub(crate) fn data(&self) -> &str {
        &self.data
    }

    #[inline]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

/// A handle to shader source stored in the [`AssetRegistry`]. Holds only an
/// [`AssetId`] — every getter takes the registry and looks the data up
/// there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShaderAsset {
    id: AssetId,
}

impl ShaderAsset {
    #[inline]
    pub(crate) fn new(id: AssetId) -> Self {
        Self { id }
    }

    #[inline]
    pub(crate) fn id(&self) -> AssetId {
        self.id
    }

    #[inline]
    pub(crate) fn get_data<'a>(&self, registry: &'a AssetRegistry) -> &'a str {
        registry.shader_data(self.id)
    }

    #[inline]
    pub(crate) fn get_path<'a>(&self, registry: &'a AssetRegistry) -> &'a Path {
        registry.shader_path(self.id)
    }
}
