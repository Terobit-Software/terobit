use glam::{Mat4, Vec3};

/// camera data used to build the view-projection matrix each frame
pub(crate) struct RenderCamera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl RenderCamera {
    /// creates a new camera looking at `target` from `position`. fov=45°, near=0.1, far=10000
    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 45f32.to_radians(),
            near: 0.1,
            far: 10000.0,
        }
    }

    /// builds the combined view-projection matrix for the given surface size.
    /// returns column-major f32 array ready for the camera uniform buffer
    pub fn build_view_proj(&self, width: u32, height: u32) -> [[f32; 4]; 4] {
        self.view_proj_mat4(width, height).to_cols_array_2d()
    }

    /// same as `build_view_proj`, but returns the `Mat4` directly instead of an array.
    /// used where the matrix needs further combining (e.g. taa's history/jitter math)
    /// before it goes to the gpu
    pub(crate) fn view_proj_mat4(&self, width: u32, height: u32) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let projection =
            Mat4::perspective_rh(self.fov, width as f32 / height as f32, self.near, self.far);
        projection * view
    }

    /// just the view matrix, no projection. used for view-space depth (shadow cascade
    /// selection) and for building each cascade's frustum corners
    pub(crate) fn view_mat4(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }
}

/// van der corput sequence in the given base. produces a low-discrepancy sequence in
/// (0, 1), used to build the halton jitter pattern below
fn van_der_corput(mut index: u32, base: u32) -> f32 {
    let mut result = 0.0;
    let mut fraction = 1.0 / base as f32;
    while index > 0 {
        result += fraction * (index % base) as f32;
        index /= base;
        fraction /= base as f32;
    }
    result
}

/// how many samples the jitter sequence cycles through before repeating. 8 is the
/// usual sweet spot for taa: enough sub-pixel coverage to resolve edges, short enough
/// that the history converges quickly after the camera stops moving
const JITTER_SEQUENCE_LENGTH: u32 = 8;

/// per-frame sub-pixel jitter offset for taa, in ndc units (so it can be added straight
/// to a clip-space position: `clip.xy += jitter * clip.w`). uses a halton(2, 3)
/// sequence, the standard low-discrepancy pattern for temporal supersampling, cycling
/// every `JITTER_SEQUENCE_LENGTH` frames
pub(crate) fn taa_jitter(frame_index: u32, width: u32, height: u32) -> [f32; 2] {
    let sample = frame_index % JITTER_SEQUENCE_LENGTH + 1;
    // halton values are in (0, 1); center on 0 so the jitter is +/- half a pixel
    let jitter_x = van_der_corput(sample, 2) - 0.5;
    let jitter_y = van_der_corput(sample, 3) - 0.5;
    [
        jitter_x * 2.0 / width as f32,
        jitter_y * 2.0 / height as f32,
    ]
}

/// camera view/projection data uploaded to the gpu every frame. holds both this
/// frame's and last frame's (unjittered) view_proj so the shader can reconstruct
/// per-pixel motion vectors, the view-only matrix (for view-space depth, used to pick
/// a shadow cascade per fragment), plus the current jitter offset applied to the
/// rasterized (but not the motion-vector) clip position
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub prev_view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub jitter: [f32; 2],
    pub _pad: [f32; 2],
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn new_has_correct_defaults() {
        let cam = RenderCamera::new(Vec3::ZERO, Vec3::X);
        assert_eq!(cam.up, Vec3::Y);
        assert!((cam.fov - 45f32.to_radians()).abs() < f32::EPSILON);
        assert!((cam.near - 0.1).abs() < f32::EPSILON);
        assert!((cam.far - 10000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn build_view_proj_all_values_finite() {
        let cam = RenderCamera::new(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO);
        let mat = cam.build_view_proj(1280, 720);
        for col in &mat {
            for val in col {
                assert!(val.is_finite(), "non-finite value in view_proj: {val}");
            }
        }
    }

    #[test]
    fn build_view_proj_differs_by_aspect_ratio() {
        let cam = RenderCamera::new(Vec3::new(0.0, 0.0, -5.0), Vec3::ZERO);
        let wide = cam.build_view_proj(1920, 1080);
        let square = cam.build_view_proj(512, 512);
        assert_ne!(wide, square);
    }
}
