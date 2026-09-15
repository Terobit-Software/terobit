use std::cell::{RefCell, RefMut};

use bevy_ecs::world::World;

/// Commands that routines can issue to the runtime through a [`RuntimeHandle`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommand {
    /// Shut the runtime down at the end of the current frame.
    Terminate,
}

/// A handle passed to routines each frame, used to issue commands back to the runtime.
/// Commands are collected during the frame and applied by the runtime once all routines have run.
pub struct RuntimeHandle<'a> {
    commands: RefCell<Vec<RuntimeCommand>>,
    // `RefCell` gives interior mutability so `world()` can hand out `&mut World`
    // even though `RuntimeRoutine::frame` only ever gets `&RuntimeHandle`.
    world: RefCell<&'a mut World>,
}

impl<'a> RuntimeHandle<'a> {
    /// Creates an empty handle for one frame.
    pub(crate) fn new(world: &'a mut World) -> Self {
        Self {
            commands: RefCell::new(Vec::new()),
            world: RefCell::new(world),
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

    /// Returns a mutable reference to the world.
    pub fn world(&self) -> RefMut<'_, World> {
        RefMut::map(self.world.borrow_mut(), |world| &mut **world)
    }
}
