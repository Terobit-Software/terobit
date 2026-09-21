#[cfg(test)]
mod tests {
    use runtime::ecs::Transform;

    #[test]
    fn new_matches_default() {
        let transform = Transform::new();
        assert_eq!(transform.get_pos(), glam::Vec3::ZERO);
        assert_eq!(transform.get_rot(), glam::Quat::IDENTITY);
        assert_eq!(transform.get_scale(), glam::Vec3::ONE);
    }

    #[test]
    fn default_transform_is_identity_matrix() {
        assert_eq!(Transform::default().to_matrix(), glam::Mat4::IDENTITY);
    }

    #[test]
    fn to_matrix_applies_scale_then_rotation_then_translation() {
        let transform = Transform::new()
            .with_pos(glam::Vec3::new(1.0, 2.0, 3.0))
            .with_rot(glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
            .with_scale(glam::Vec3::new(2.0, 2.0, 2.0));

        // (1, 0, 0) -> scaled (2, 0, 0) -> rotated 90deg about Z (0, 2, 0) -> translated (1, 4, 3)
        let point = transform.to_matrix().transform_point3(glam::Vec3::X);
        assert!(point.abs_diff_eq(glam::Vec3::new(1.0, 4.0, 3.0), 1e-5));
    }

    #[test]
    fn with_pos_sets_position() {
        let transform = Transform::new().with_pos(glam::Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.get_pos(), glam::Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn set_pos_mutates_position() {
        let mut transform = Transform::new();
        transform.set_pos(glam::Vec3::new(4.0, 5.0, 6.0));
        assert_eq!(transform.get_pos(), glam::Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn with_rot_sets_rotation() {
        let rotation = glam::Quat::from_rotation_y(1.0);
        let transform = Transform::new().with_rot(rotation);
        assert_eq!(transform.get_rot(), rotation);
    }

    #[test]
    fn set_rot_mutates_rotation() {
        let mut transform = Transform::new();
        let rotation = glam::Quat::from_rotation_x(0.5);
        transform.set_rot(rotation);
        assert_eq!(transform.get_rot(), rotation);
    }

    #[test]
    fn with_scale_sets_scale() {
        let transform = Transform::new().with_scale(glam::Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(transform.get_scale(), glam::Vec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn set_scale_mutates_scale() {
        let mut transform = Transform::new();
        transform.set_scale(glam::Vec3::new(0.5, 0.5, 0.5));
        assert_eq!(transform.get_scale(), glam::Vec3::new(0.5, 0.5, 0.5));
    }

    #[test]
    fn builder_methods_chain_together() {
        let transform = Transform::new()
            .with_pos(glam::Vec3::new(1.0, 0.0, 0.0))
            .with_rot(glam::Quat::IDENTITY)
            .with_scale(glam::Vec3::new(1.0, 1.0, 1.0));

        assert_eq!(transform.get_pos(), glam::Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(transform.get_rot(), glam::Quat::IDENTITY);
        assert_eq!(transform.get_scale(), glam::Vec3::new(1.0, 1.0, 1.0));
    }
}
