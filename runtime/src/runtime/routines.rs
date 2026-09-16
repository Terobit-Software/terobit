use crate::runtime::handle::RuntimeHandle;

/// A routine that can be executed during the runtime.
pub trait RuntimeRoutine {
    /// The priority of the routine. Two same-priority routines will be executed in the order they were added.
    /// A lower priority value means the routine will be executed earlier.
    fn priority(&self) -> u8;
    /// The frame of the routine.
    fn frame(&self, handle: &RuntimeHandle);
    /// The name of the routine.
    fn name(&self) -> &str;
    /// Returns whether the routine is enabled and should be run.
    fn is_enabled(&self) -> bool {
        true
    }
}

pub(crate) struct RoutineManager {
    routines: Vec<Box<dyn RuntimeRoutine>>,
}

impl RoutineManager {
    pub fn new() -> Self {
        Self {
            routines: Vec::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, routine: Box<dyn RuntimeRoutine>) {
        self.routines.push(routine);
        self.routines.sort_by_key(|r| r.priority());
    }

    pub fn run(&self, handle: &RuntimeHandle) {
        for routine in &self.routines {
            if routine.is_enabled() {
                routine.frame(handle);
            }
        }
    }
}
