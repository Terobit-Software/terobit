use thiserror::Error;
use winit::event_loop::{self, EventLoop};

#[derive(Error, Debug)]
pub enum RuntimeInitError {
    #[error("failed to initialize")]
    UnknownInitFail,
    #[error("wgpu failed")]
    WgpuFailed,
    #[error("winit event loop failed")]
    WinitEventLoopFailed,
}

pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn start(&self) -> Result<(), RuntimeInitError> {
        let event_loop = EventLoop::new();

        if let Ok(event) = event_loop {
        } else {
            return Err(RuntimeInitError::WinitEventLoopFailed);
        }

        Ok(())
    }
}
