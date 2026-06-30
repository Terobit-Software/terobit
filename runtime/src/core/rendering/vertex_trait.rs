/// trait for vertex types that can be converted to a gpu-ready packed form
pub(crate) trait VertexTrait {
    /// the packed, `#[repr(C)]` representation for uploading to the gpu
    type Raw: bytemuck::Pod + bytemuck::Zeroable;

    /// converts this vertex to its packed gpu form
    fn to_raw(&self) -> Self::Raw;

    /// returns the wgpu vertex buffer layout descriptor for this vertex type
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}
