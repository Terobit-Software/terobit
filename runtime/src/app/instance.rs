use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// Per-instance data uploaded alongside the shared cube mesh: just a model
/// matrix for now, one row per instance drawn.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub(crate) fn from_model(model: Mat4) -> Self {
        Self {
            model: model.to_cols_array_2d(),
        }
    }
}
