#[cfg(test)]
mod tests {
    use runtime::prelude::*;

    struct NoOpRoutine;

    impl RuntimeRoutine for NoOpRoutine {
        fn priority(&self) -> u8 {
            0
        }

        fn frame(&self, _handle: &RuntimeHandle) {}

        fn name(&self) -> &str {
            "no_op"
        }

        fn is_enabled(&self) -> bool {
            true
        }
    }

    #[test]
    fn prelude_exposes_runtime_and_routines() {
        let mut runtime = Runtime::new();
        runtime.add_routine(NoOpRoutine);
    }

    #[test]
    fn prelude_glam_types_work_with_transform() {
        let transform = Transform::new()
            .with_pos(Vec3::new(1.0, 2.0, 3.0))
            .with_rot(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0));
        assert_eq!(transform.get_pos(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(Mesh::CUBE.asset_id(), Mesh::new(0).asset_id());
    }
}
