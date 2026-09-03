use std::sync::Arc;

/// The main application state.
pub(crate) struct AppState {
    instance: wgpu::Instance,
    window: crate::app::window::Window,
}

impl AppState {
    /// Creates a new instance of [`AppState`].
    pub(crate) fn new() -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor::default()),
            window: crate::app::window::Window::new(),
        }
    }
}
