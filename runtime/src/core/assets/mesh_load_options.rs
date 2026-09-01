/// options for `RuntimeContext::load_mesh_with`. the default loads the mesh untouched
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshLoadOptions {
    /// keep roughly this fraction of the original triangles (0.0..=1.0).
    /// `None` skips simplification entirely, which is the default
    pub simplify_ratio: Option<f32>,
    /// max allowed simplification error as a fraction of the mesh extents,
    /// only used when `simplify_ratio` is set
    pub simplify_target_error: f32,
}

impl Default for MeshLoadOptions {
    fn default() -> Self {
        Self {
            simplify_ratio: None,
            simplify_target_error: 0.01,
        }
    }
}

impl MeshLoadOptions {
    /// simplify on load, keeping roughly `ratio` of the original triangles
    pub fn simplified(ratio: f32) -> Self {
        Self {
            simplify_ratio: Some(ratio),
            ..Self::default()
        }
    }

    // bit patterns instead of raw f32s so the mesh cache key stays Eq + Hash,
    // same trick the physics shape cache uses for scale/mass
    pub(crate) fn cache_key_bits(&self) -> Option<(u32, u32)> {
        self.simplify_ratio
            .map(|ratio| (ratio.to_bits(), self.simplify_target_error.to_bits()))
    }
}
