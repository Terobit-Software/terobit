pub trait VertexTrait {
    type Raw: bytemuck::Pod + bytemuck::Zeroable;
    
    fn to_raw(&self) -> Self::Raw;
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}