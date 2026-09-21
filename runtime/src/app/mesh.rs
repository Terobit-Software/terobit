use std::collections::{HashMap, HashSet};
use std::ops::Range;

use tracing::{info, warn};
use wgpu::util::DeviceExt;

use crate::assets::{AssetId, AssetRegistry, Vertex};

/// A mesh's geometry uploaded to the GPU: immutable vertex + index buffers
/// shared by every instance drawn from it.
pub(crate) struct GpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl GpuMesh {
    fn upload(device: &wgpu::Device, id: AssetId, verts: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("mesh {id} vertex buffer")),
            contents: bytemuck::cast_slice(verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("mesh {id} index buffer")),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    /// Binds this mesh at vertex slot 0 and draws it once per instance in
    /// `instances`, which index into the instance buffer already bound at slot 1.
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>, instances: Range<u32>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, instances);
    }
}

/// Meshes already uploaded to the GPU, keyed by their asset id. Meshes are
/// uploaded the first time something asks to draw them.
pub(crate) struct GpuMeshCache {
    meshes: HashMap<AssetId, GpuMesh>,
    /// ids that were asked for but don't exist in the registry, so the warning
    /// is logged once instead of every frame
    missing: HashSet<AssetId>,
}

impl GpuMeshCache {
    pub(crate) fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            missing: HashSet::new(),
        }
    }

    /// Uploads mesh `id` from the registry if it isn't on the GPU yet.
    /// Returns whether the mesh is now available to [`Self::get`].
    pub(crate) fn ensure_uploaded(
        &mut self,
        device: &wgpu::Device,
        registry: &AssetRegistry,
        id: AssetId,
    ) -> bool {
        if self.meshes.contains_key(&id) {
            return true;
        }

        let Some(asset) = registry.get_mesh(id) else {
            if self.missing.insert(id) {
                warn!("Mesh asset {} isn't loaded, so it can't be drawn", id);
            }
            return false;
        };

        let gpu_mesh = GpuMesh::upload(device, id, asset.verts(registry), asset.indices(registry));
        info!(
            "Uploaded mesh {} to the GPU ({} vertices, {} indices)",
            id,
            asset.verts(registry).len(),
            gpu_mesh.index_count
        );
        self.meshes.insert(id, gpu_mesh);

        true
    }

    #[inline]
    pub(crate) fn get(&self, id: AssetId) -> Option<&GpuMesh> {
        self.meshes.get(&id)
    }
}
