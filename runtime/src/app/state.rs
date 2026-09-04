use crate::app::window::Window;
use thiserror::Error;
use tracing::info;
use winit::event_loop::ActiveEventLoop;

#[derive(Error, Debug)]
pub(crate) enum AppStateError {
    #[error("failed to create window")]
    WindowError(#[from] crate::app::window::WindowError),
    #[error("failed to create wgpu adapter")]
    WgpuAdapterError(#[from] wgpu::RequestAdapterError),
    #[error("failed to create wgpu device")]
    WgpuDeviceError(#[from] wgpu::RequestDeviceError),
}

/// The main application state.
pub(crate) struct AppState {
    instance: wgpu::Instance,
    window: Window,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl AppState {
    /// Creates a new instance of [`AppState`].
    pub(crate) async fn new(event_loop: &ActiveEventLoop) -> Result<Self, AppStateError> {
        // create the window
        let window = Window::new(event_loop)?;
        window.get_window().set_title("Terobit Client");

        // create the wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(event_loop.owned_display_handle()),
        ));
        info!("New wgpu instance successfully created");

        // create the wgpu adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        info!("New wgpu adapter successfully created");

        // create the wgpu device
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        info!("New wgpu device and queue successfully created");

        // print device info
        let info = device.adapter_info();
        info!("Device info: {:?}", info);

        Ok(Self {
            instance,
            window,
            adapter,
            device,
            queue,
        })
    }

    pub(crate) fn run(&self) {}
}
