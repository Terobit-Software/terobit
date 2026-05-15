use std::collections::HashMap;

/// Uploaded mesh data sitting on the GPU
pub(crate) struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}