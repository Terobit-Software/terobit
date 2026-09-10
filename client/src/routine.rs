use rand::RngExt;
use runtime::handle::RuntimeHandle;
use runtime::routines::RuntimeRoutine;

pub struct Routine {}

impl RuntimeRoutine for Routine {
    fn priority(&self) -> u8 {
        0
    }

    fn frame(&self, _handle: &RuntimeHandle) {
        let mut rng = rand::rng();
        let num: u16 = rng.random();
        // println!("{}", num);
        if num == 0 {
            _handle.terminate();
        }
    }

    fn name(&self) -> &str {
        "test"
    }

    fn is_enabled(&self) -> bool {
        true
    }
}
