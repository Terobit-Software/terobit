use std::sync::Arc;
use thiserror::Error;
use tracing::info;
use winit::error::OsError;
use winit::event_loop::ActiveEventLoop;

/// Represents a window that can be used for rendering
pub(crate) struct Window {
    window: Arc<winit::window::Window>,
}

#[derive(Error, Debug)]
/// Errors that can occur when creating a window
pub(crate) enum WindowError {
    #[error("failed to create window")]
    CreateWindowError(#[from] OsError),
}

impl Window {
    /// Creates a new window using the given event loop
    pub fn new(event_loop: &ActiveEventLoop) -> Result<Self, WindowError> {
        let window = event_loop.create_window(winit::window::Window::default_attributes())?;

        info!("New window successfully created with id {:?}", window.id());

        Ok(Self {
            window: Arc::new(window),
        })
    }

    /// Returns a reference to the window
    pub fn get_window(&self) -> &winit::window::Window {
        &self.window
    }

    /// Returns an owned handle to the window
    pub fn get_window_arc(&self) -> Arc<winit::window::Window> {
        Arc::clone(&self.window)
    }
}
