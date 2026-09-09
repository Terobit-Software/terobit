use std::cell::RefCell;

/// Commands that routines can issue to the runtime through a [`RuntimeHandle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommand {
    /// Shut the runtime down at the end of the current frame.
    Terminate,
}

/// A handle passed to routines each frame, used to issue commands back to the runtime.
/// Commands are collected during the frame and applied by the runtime once all routines have run.
pub struct RuntimeHandle {
    commands: RefCell<Vec<RuntimeCommand>>,
}

impl RuntimeHandle {
    /// Creates an empty handle for one frame.
    pub(crate) fn new() -> Self {
        Self {
            commands: RefCell::new(Vec::new()),
        }
    }

    /// Requests that the runtime shut down at the end of the current frame.
    pub fn terminate(&self) {
        self.commands.borrow_mut().push(RuntimeCommand::Terminate);
    }

    /// Consumes the handle, yielding the commands issued during the frame.
    pub(crate) fn take_commands(self) -> Vec<RuntimeCommand> {
        self.commands.into_inner()
    }
}
