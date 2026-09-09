/// A routine that can be executed during the runtime.
pub trait RuntimeRoutine {
    /// The priority of the routine. Two same-priority routines will be executed in the order they were added.
    /// A lower priority value means the routine will be executed earlier.
    fn priority(&self) -> u8;
    /// The frame of the routine.
    fn frame(&self);
    /// The name of the routine.
    fn name(&self) -> &str;
    /// Returns whether the routine is enabled and should be run.
    fn is_enabled(&self) -> bool;
}
