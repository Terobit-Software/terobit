use crate::app::RuntimeApp;
use thiserror::Error;
use winit::event_loop::{self, EventLoop};

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
        // create the event loop
        let event_loop = EventLoop::new();

        // check it
        if let Ok(event) = event_loop {
            // set the control flow to poll
            event.set_control_flow(event_loop::ControlFlow::Poll);

            let app = RuntimeApp::new();
            event
                .run(move |_event, _target| {
                    app.run();
                })
                .map_err(|_| RuntimeInitError::WinitEventLoopFailed)?;
        } else {
            // return an error if event loop fails for some reason
            return Err(RuntimeInitError::WinitEventLoopFailed);
        }

        Ok(())
    }
}
