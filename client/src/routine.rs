use runtime::routines::RuntimeRoutine;

pub struct Routine {}

impl RuntimeRoutine for Routine {
    fn priority(&self) -> u8 {
        0
    }

    fn frame(&self) {}

    fn name(&self) -> &str {
        "test"
    }

    fn is_enabled(&self) -> bool {
        true
    }
}
