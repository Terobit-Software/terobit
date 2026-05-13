use crate::rendering::{Vertex, VertexTrait};
use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

pub(crate) struct RenderMesh<V: VertexTrait> {
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    _phantom: std::marker::PhantomData<V>,
}

impl<V: VertexTrait> RenderMesh<V> {
    pub fn new(device: &wgpu::Device, vertices: &[V]) -> Self {
        let raw: Vec<V::Raw> = vertices.iter().map(|v| v.to_raw()).collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&raw),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl RenderMesh<Vertex> {
    pub fn cube(device: &wgpu::Device, position: Vec3, size: f32) -> Self {
        let h = size / 2.0;
        let x = position.x;
        let y = position.y;
        let z = position.z;

        #[rustfmt::skip]
        let vertices = vec![
            // Front face (normal: +Z)
            Vertex::new(Vec3::new(x - h, y - h, z + h), Vec2::new(0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(x + h, y - h, z + h), Vec2::new(1.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(x + h, y + h, z + h), Vec2::new(1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(x - h, y - h, z + h), Vec2::new(0.0, 1.0), Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(x + h, y + h, z + h), Vec2::new(1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(x - h, y + h, z + h), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),

            // Back face (normal: -Z)
            Vertex::new(Vec3::new(x + h, y - h, z - h), Vec2::new(0.0, 1.0), Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(Vec3::new(x - h, y - h, z - h), Vec2::new(1.0, 1.0), Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(Vec3::new(x - h, y + h, z - h), Vec2::new(1.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(Vec3::new(x + h, y - h, z - h), Vec2::new(0.0, 1.0), Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(Vec3::new(x - h, y + h, z - h), Vec2::new(1.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
            Vertex::new(Vec3::new(x + h, y + h, z - h), Vec2::new(0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),

            // Left face (normal: -X)
            Vertex::new(Vec3::new(x - h, y - h, z - h), Vec2::new(0.0, 1.0), Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y - h, z + h), Vec2::new(1.0, 1.0), Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y + h, z + h), Vec2::new(1.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y - h, z - h), Vec2::new(0.0, 1.0), Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y + h, z + h), Vec2::new(1.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y + h, z - h), Vec2::new(0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)),

            // Right face (normal: +X)
            Vertex::new(Vec3::new(x + h, y - h, z + h), Vec2::new(0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y - h, z - h), Vec2::new(1.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y + h, z - h), Vec2::new(1.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y - h, z + h), Vec2::new(0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y + h, z - h), Vec2::new(1.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y + h, z + h), Vec2::new(0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),

            // Top face (normal: +Y)
            Vertex::new(Vec3::new(x - h, y + h, z + h), Vec2::new(0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y + h, z + h), Vec2::new(1.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y + h, z - h), Vec2::new(1.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y + h, z + h), Vec2::new(0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y + h, z - h), Vec2::new(1.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y + h, z - h), Vec2::new(0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),

            // Bottom face (normal: -Y)
            Vertex::new(Vec3::new(x - h, y - h, z - h), Vec2::new(0.0, 1.0), Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y - h, z - h), Vec2::new(1.0, 1.0), Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y - h, z + h), Vec2::new(1.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y - h, z - h), Vec2::new(0.0, 1.0), Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(Vec3::new(x + h, y - h, z + h), Vec2::new(1.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
            Vertex::new(Vec3::new(x - h, y - h, z + h), Vec2::new(0.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
        ];

        Self::new(device, &vertices)
    }
}

impl<V: VertexTrait> RenderMesh<V> {
    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}
