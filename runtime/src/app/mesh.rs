use bytemuck::{Pod, Zeroable};

/// A single vertex: position and face normal, both in model space.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

/// Unit cube (extents -0.5..0.5 on each axis), 4 duplicated vertices per face
/// so each face gets its own flat normal instead of smoothed corner normals.
pub(crate) const CUBE_VERTICES: [Vertex; 24] = [
    // +X
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
    },
    // -X
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    // +Y
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
    },
    // -Y
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
    },
    // +Z
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    // -Z
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
];

/// Two CCW-wound triangles per face, indexing into [`CUBE_VERTICES`].
pub(crate) const CUBE_INDICES: [u16; 36] = [
    0, 1, 2, 0, 2, 3, // +X
    4, 5, 6, 4, 6, 7, // -X
    8, 9, 10, 8, 10, 11, // +Y
    12, 13, 14, 12, 14, 15, // -Y
    16, 17, 18, 16, 18, 19, // +Z
    20, 21, 22, 20, 22, 23, // -Z
];
