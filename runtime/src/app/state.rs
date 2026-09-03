use crate::app::window::Window;
use thiserror::Error;
use winit::event_loop::ActiveEventLoop;

#[derive(Error, Debug)]
pub(crate) enum AppStateError {
    #[error("failed to create window")]
    WindowError(#[from] crate::app::window::WindowError),
}

/// The main application state.
pub(crate) struct AppState {
    instance: wgpu::Instance,
    window: Window,
}

impl AppState {
    /// Creates a new instance of [`AppState`].
    pub(crate) fn new(event_loop: &ActiveEventLoop) -> Result<Self, AppStateError> {
        let window = Window::new(event_loop)?;

        window.get_window().set_title("Terobit Client");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(event_loop.owned_display_handle()),
        ));

        Ok(Self { instance, window })
    }

    pub(crate) fn run(&self) {}
}
