use std::sync::Arc;
use thiserror::Error;
use winit::error::OsError;
use winit::event_loop::ActiveEventLoop;

pub(crate) struct Window {
    window: Arc<winit::window::Window>,
}

#[derive(Error, Debug)]
pub(crate) enum WindowError {
    #[error("failed to create window")]
    CreateWindowError(#[from] OsError),
}

impl Window {
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, WindowError> {
        let window = event_loop.create_window(winit::window::Window::default_attributes())?;

        Ok(Self {
            window: Arc::new(window),
        })
    }

    pub fn get_window(&self) -> &winit::window::Window {
        &self.window
    }
}
