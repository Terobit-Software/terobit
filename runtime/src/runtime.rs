use crate::app::RuntimeApp;
use thiserror::Error;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod routines;
use routines::RuntimeRoutine;

pub mod handle;

#[derive(Error, Debug)]
/// Errors that can occur during runtime initialization.
pub enum RuntimeInitError {
    #[error("failed to initialize")]
    UnknownInitFail,
    #[error("wgpu failed")]
    WgpuFailed,
    #[error("winit event loop failed")]
    WinitEventLoopFailed,
    #[error("failed to add routine")]
    FailedToAddRoutine,
}

/// The runtime event loop.
pub struct Runtime {
    app: RuntimeApp,
}

impl Runtime {
    /// Creates a new instance of [`Runtime`].
    pub fn new() -> Self {
        Self {
            app: RuntimeApp::new(),
        }
    }

    /// Starts the runtime event loop.
    pub fn start(&mut self) -> Result<(), RuntimeInitError> {
        let event_loop = EventLoop::new().map_err(|_| RuntimeInitError::WinitEventLoopFailed)?;

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
            .run_app(&mut self.app)
            .map_err(|_| RuntimeInitError::WinitEventLoopFailed)?;

        Ok(())
    }

    /// Adds a routine to be run every frame. Routines added before [`Runtime::start`]
    /// are queued and installed once the app state exists.
    pub fn add_routine(&mut self, routine: impl RuntimeRoutine + 'static) {
        self.app.queue_routine(Box::new(routine));
    }
}
