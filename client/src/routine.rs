use rand::RngExt;
use runtime::ecs::{Mesh, Transform};
use runtime::handle::RuntimeHandle;
use runtime::routines::RuntimeRoutine;

pub struct Routine {}

impl RuntimeRoutine for Routine {
    #[inline]
    fn priority(&self) -> u8 {
        0
    }

    fn frame(&self, _handle: &RuntimeHandle) {
        let mut rng = rand::rng();
        // let num: u8 = rng.random();
        //_handle.set_title(&num.to_string());
        // if num == 0 {
        // scatter cubes around the origin, each at a random orientation so
        // they read as 3D
        for _ in 0..100 {
            let transform = Transform::new()
                .with_pos(glam::Vec3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-0.7..0.7),
                    rng.random_range(-1.0..1.0),
                ))
                .with_rot(glam::Quat::from_euler(
                    glam::EulerRot::XYZ,
                    rng.random_range(0.0..std::f32::consts::TAU),
                    rng.random_range(0.0..std::f32::consts::TAU),
                    rng.random_range(0.0..std::f32::consts::TAU),
                ))
                .with_scale(glam::Vec3::splat(0.3));
            _handle.world().spawn((Mesh::CUBE, transform));
        }
    }

    #[inline]
    fn name(&self) -> &str {
        "test"
    }

    #[inline]
    fn is_enabled(&self) -> bool {
        true
    }
}
