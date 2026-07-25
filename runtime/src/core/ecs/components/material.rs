use glam::Vec4;

/// per-entity color material. optional, defaults to white if not present
#[derive(bevy_ecs::component::Component, Clone, Copy)]
pub struct Material {
    /// rgba color, each channel 0.0-1.0
    pub color: [f32; 4],
}

impl Material {
    /// creates a material with the given rgba color
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            color: [r, g, b, a],
        }
    }

    /// creates a material from a Vec4 (x=r, y=g, z=b, w=a)
    pub fn from_vec4(v: Vec4) -> Self {
        Self {
            color: [v.x, v.y, v.z, v.w],
        }
    }

    pub fn white() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
    }
    pub fn red() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }
    pub fn green() -> Self {
        Self::new(0.0, 1.0, 0.0, 1.0)
    }
    pub fn blue() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }
    pub fn black() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::white()
    }
}
