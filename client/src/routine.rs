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
        let num: u8 = rng.random();
        //_handle.set_title(&num.to_string());
        if num == 0 {
            _handle.world().spawn((Mesh::CUBE, Transform::default()));
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
