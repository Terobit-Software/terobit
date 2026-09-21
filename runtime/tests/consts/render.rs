#[cfg(test)]
mod tests {
    use runtime::consts::render::*;

    #[test]
    fn depth_format_is_depth32_float() {
        assert_eq!(DEPTH_FORMAT, wgpu::TextureFormat::Depth32Float);
    }

    #[test]
    fn camera_fov_is_forty_five_degrees() {
        assert_eq!(CAMERA_FOV_Y_RADIANS, std::f32::consts::FRAC_PI_4);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn camera_near_is_closer_than_far() {
        assert!(CAMERA_NEAR < CAMERA_FAR);
    }

    #[test]
    fn camera_position_is_offset_along_z() {
        assert_eq!(CAMERA_POSITION, glam::Vec3::new(0.0, 0.0, 3.0));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn instance_buffer_starts_with_room_for_instances() {
        assert!(INITIAL_INSTANCE_CAPACITY > 0);
    }
}
