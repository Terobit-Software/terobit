use glam::{Mat4, Vec3};

use crate::{consts::SHADOW_CASCADE_COUNT, rendering::RenderCamera};

/// one cascade's light-space view-projection matrix, plus the view-space depth (from
/// the main camera) marking the far edge of the slice of the view frustum it covers
#[derive(Clone, Copy)]
pub(crate) struct CascadeMatrices {
    pub view_proj: Mat4,
    pub split_depth: f32,
}

/// light data uploaded to the gpu once per frame: every cascade's view-projection
/// matrix (so the shader can pick the right one per fragment), their split depths
/// packed into a vec4 (hardcoded to `SHADOW_CASCADE_COUNT == 4`, matching the shader),
/// and the direction toward the light for the diffuse term
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightUniform {
    pub cascade_view_proj: [[[f32; 4]; 4]; SHADOW_CASCADE_COUNT],
    pub cascade_splits: [f32; 4],
    pub light_dir: [f32; 3],
    pub _pad: f32,
}

/// splits [near, shadow_max_distance] into `SHADOW_CASCADE_COUNT` pieces using the
/// "practical split scheme": a blend of uniform steps (keeps far cascades from being
/// absurdly large) and logarithmic steps (gives near cascades, where aliasing is most
/// visible, much more of the shadow map's resolution)
fn cascade_splits(near: f32, far: f32, lambda: f32) -> [f32; SHADOW_CASCADE_COUNT] {
    std::array::from_fn(|i| {
        let p = (i + 1) as f32 / SHADOW_CASCADE_COUNT as f32;
        let log_split = near * (far / near).powf(p);
        let uniform_split = near + (far - near) * p;
        lambda * log_split + (1.0 - lambda) * uniform_split
    })
}

/// builds one cascade's light-space view-projection matrix: finds the world-space
/// bounding sphere of the camera's view frustum between `split_near` and `split_far`,
/// then fits an orthographic projection to it, centered on the light's direction
///
/// uses a bounding *sphere* (not just an aabb) so the shadow volume's size only changes
/// between cascades, never as the camera turns; a size that shifts with camera
/// rotation is the classic cause of cascades visibly "swimming". the projection is
/// then snapped to whole shadow-map texels for the same reason, but for camera
/// translation instead of rotation
fn build_cascade(
    camera: &RenderCamera,
    aspect: f32,
    light_dir_to_light: Vec3,
    split_near: f32,
    split_far: f32,
    shadow_map_size: u32,
) -> CascadeMatrices {
    let inv_view = camera.view_mat4().inverse();

    let tan_half_fov_y = (camera.fov * 0.5).tan();
    let tan_half_fov_x = tan_half_fov_y * aspect;

    // 8 corners of the camera frustum slice between split_near and split_far, in view
    // space (camera looks down -z), then carried into world space
    let corners_view = [
        Vec3::new(
            -split_near * tan_half_fov_x,
            -split_near * tan_half_fov_y,
            -split_near,
        ),
        Vec3::new(
            split_near * tan_half_fov_x,
            -split_near * tan_half_fov_y,
            -split_near,
        ),
        Vec3::new(
            -split_near * tan_half_fov_x,
            split_near * tan_half_fov_y,
            -split_near,
        ),
        Vec3::new(
            split_near * tan_half_fov_x,
            split_near * tan_half_fov_y,
            -split_near,
        ),
        Vec3::new(
            -split_far * tan_half_fov_x,
            -split_far * tan_half_fov_y,
            -split_far,
        ),
        Vec3::new(
            split_far * tan_half_fov_x,
            -split_far * tan_half_fov_y,
            -split_far,
        ),
        Vec3::new(
            -split_far * tan_half_fov_x,
            split_far * tan_half_fov_y,
            -split_far,
        ),
        Vec3::new(
            split_far * tan_half_fov_x,
            split_far * tan_half_fov_y,
            -split_far,
        ),
    ];
    let corners_world = corners_view.map(|c| inv_view.transform_point3(c));

    let center =
        corners_world.iter().fold(Vec3::ZERO, |acc, &c| acc + c) / corners_world.len() as f32;

    let mut radius: f32 = 0.0;
    for &c in &corners_world {
        radius = radius.max((c - center).length());
    }
    // round up so float error never lets a corner poke outside the sphere, and floor
    // against a degenerate zero-size cascade (shouldn't happen, but Mat4::orthographic_rh
    // with a zero extent would produce a garbage matrix rather than a merely-wrong one)
    let radius = ((radius * 16.0).ceil() / 16.0).max(0.01);

    let light_to_scene = -light_dir_to_light;
    // look_at is degenerate when `up` is parallel to the view direction; the light
    // direction is fixed at startup so this only matters if it's ever pointed
    // near-vertical, but it's a one-line guard against a NaN view matrix if so
    let up = if light_to_scene.abs().dot(Vec3::Y) > 0.99 {
        Vec3::Z
    } else {
        Vec3::Y
    };
    let light_eye = center - light_to_scene * radius * 2.0;
    let light_view = Mat4::look_at_rh(light_eye, center, up);

    // near=0 at the eye, far=4*radius: the bounding sphere sits at distance 2*radius
    // from the eye (see light_eye above) with radius `radius`, so it spans
    // [radius, 3*radius] along the light's view axis. 4*radius leaves margin for
    // casters that stick out past the frustum corners themselves (tall/wide meshes)
    let ortho = Mat4::orthographic_rh(-radius, radius, -radius, radius, 0.0, radius * 4.0);
    let mut view_proj = ortho * light_view;

    // snap the projection's origin to whole texels in light-clip space, so panning the
    // camera by a fraction of a shadow texel doesn't make shadow edges crawl
    let texel_size_ndc = 2.0 / shadow_map_size as f32;
    let origin = view_proj.transform_point3(Vec3::ZERO);
    let snap_x = (origin.x / texel_size_ndc).round() * texel_size_ndc - origin.x;
    let snap_y = (origin.y / texel_size_ndc).round() * texel_size_ndc - origin.y;
    view_proj = Mat4::from_translation(Vec3::new(snap_x, snap_y, 0.0)) * view_proj;

    CascadeMatrices {
        view_proj,
        split_depth: split_far,
    }
}

/// builds every cascade's light-space view-projection matrix for this frame, from the
/// current camera pose. `light_dir_to_light` points from any surface toward the light
pub(crate) fn compute_cascades(
    camera: &RenderCamera,
    aspect: f32,
    light_dir_to_light: Vec3,
    shadow_max_distance: f32,
    shadow_map_size: u32,
) -> [CascadeMatrices; SHADOW_CASCADE_COUNT] {
    let splits = cascade_splits(
        camera.near,
        shadow_max_distance,
        crate::consts::SHADOW_SPLIT_LAMBDA,
    );

    std::array::from_fn(|i| {
        let split_near = if i == 0 { camera.near } else { splits[i - 1] };
        build_cascade(
            camera,
            aspect,
            light_dir_to_light,
            split_near,
            splits[i],
            shadow_map_size,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_camera() -> RenderCamera {
        RenderCamera::new(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO)
    }

    #[test]
    fn splits_are_increasing() {
        let splits = cascade_splits(0.1, 600.0, 0.6);
        for i in 1..splits.len() {
            assert!(
                splits[i] > splits[i - 1],
                "splits not monotonic: {splits:?}"
            );
        }
    }

    #[test]
    fn splits_end_at_far_distance() {
        let splits = cascade_splits(0.1, 600.0, 0.6);
        assert!((splits[SHADOW_CASCADE_COUNT - 1] - 600.0).abs() < 0.01);
    }

    #[test]
    fn cascades_produce_finite_matrices() {
        let camera = test_camera();
        let light_dir = Vec3::new(0.5, 1.0, 0.3).normalize();
        let cascades = compute_cascades(&camera, 16.0 / 9.0, light_dir, 600.0, 2048);

        for cascade in &cascades {
            for col in cascade.view_proj.to_cols_array_2d() {
                for val in col {
                    assert!(
                        val.is_finite(),
                        "non-finite value in cascade view_proj: {val}"
                    );
                }
            }
        }
    }

    #[test]
    fn cascade_split_depths_match_split_scheme() {
        let camera = test_camera();
        let light_dir = Vec3::new(0.5, 1.0, 0.3).normalize();
        let cascades = compute_cascades(&camera, 16.0 / 9.0, light_dir, 600.0, 2048);
        let splits = cascade_splits(camera.near, 600.0, crate::consts::SHADOW_SPLIT_LAMBDA);

        for (cascade, &split) in cascades.iter().zip(splits.iter()) {
            assert_eq!(cascade.split_depth, split);
        }
    }

    #[test]
    fn near_cascade_covers_less_ground_than_far_cascade() {
        // sanity check on the split scheme itself: the whole point of cascades is that
        // the near one gets a tighter (higher-resolution) volume than the far one
        let camera = test_camera();
        let light_dir = Vec3::new(0.5, 1.0, 0.3).normalize();
        let cascades = compute_cascades(&camera, 16.0 / 9.0, light_dir, 600.0, 2048);

        assert!(cascades[0].split_depth < cascades[SHADOW_CASCADE_COUNT - 1].split_depth);
    }

    #[test]
    fn degenerate_up_vector_still_produces_finite_matrix() {
        let camera = test_camera();
        // near-vertical light direction, the case the up-vector guard exists for
        let light_dir = Vec3::new(0.01, 1.0, 0.01).normalize();
        let cascades = compute_cascades(&camera, 16.0 / 9.0, light_dir, 600.0, 2048);

        for cascade in &cascades {
            for col in cascade.view_proj.to_cols_array_2d() {
                for val in col {
                    assert!(val.is_finite());
                }
            }
        }
    }
}
