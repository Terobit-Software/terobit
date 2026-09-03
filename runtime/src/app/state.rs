use crate::app::window::Window;
use std::sync::Arc;
use thiserror::Error;
use winit::error::OsError;
use winit::event_loop;

#[derive(Error, Debug)]
pub(crate) enum AppStateError {
    #[error("failed to create window")]
    WindowError(#[from] crate::app::window::WindowError),
}

/// The main application state.
pub(crate) struct AppState {
    instance: wgpu::Instance,
    window: crate::app::window::Window,
}

impl AppState {
    /// Creates a new instance of [`AppState`].
    pub(crate) async fn new(
        event_loop: winit::event_loop::EventLoop<()>,
    ) -> Result<Self, AppStateError> {
        let window = Window::new(&event_loop)?;

        // no display handle here, winit 0.29 can't hand out an owned one and
        // wgpu only needs it for gles/wayland anyway
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        Ok(Self { instance, window })
    }

    pub(crate) fn run(&self) {}
}
