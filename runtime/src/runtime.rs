use crate::app::RuntimeApp;
use thiserror::Error;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Error, Debug)]
/// Errors that can occur during runtime initialization.
pub enum RuntimeInitError {
    #[error("failed to initialize")]
    UnknownInitFail,
    #[error("wgpu failed")]
    WgpuFailed,
    #[error("winit event loop failed")]
    WinitEventLoopFailed,
}

/// The runtime event loop.
pub struct Runtime {}

impl Runtime {
    /// Creates a new instance of [`Runtime`].
    pub fn new() -> Self {
        Self {}
    }

    /// Starts the runtime event loop.
    pub fn start(&self) -> Result<(), RuntimeInitError> {
        let event_loop = EventLoop::new().map_err(|_| RuntimeInitError::WinitEventLoopFailed)?;

        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = RuntimeApp::new();
        event_loop
            .run_app(&mut app)
            .map_err(|_| RuntimeInitError::WinitEventLoopFailed)?;

        Ok(())
    }
}
