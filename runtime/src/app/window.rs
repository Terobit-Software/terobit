use std::sync::Arc;
use thiserror::Error;
use winit::{error::OsError, window};

pub(crate) struct Window {
    window: Arc<winit::window::Window>,
}

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("failed to create window")]
    CreateWindowError(#[from] OsError),
}

impl Window {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, WindowError> {
        Ok(Self {
            window: Arc::new(winit::window::Window::new(event_loop)?),
        })
    }

    pub fn get_window(&self) -> &winit::window::Window {
        &self.window
    }
}
