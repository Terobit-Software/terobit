use crate::app::window::Window;
use thiserror::Error;
use tracing::{info, warn};
use winit::event_loop::ActiveEventLoop;

#[derive(Error, Debug)]
pub(crate) enum AppStateError {
    #[error("failed to create window")]
    WindowError(#[from] crate::app::window::WindowError),
    #[error("failed to create wgpu adapter")]
    WgpuAdapterError(#[from] wgpu::RequestAdapterError),
    #[error("failed to create wgpu device")]
    WgpuDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("failed to create wgpu surface")]
    WgpuSurfaceError(#[from] wgpu::CreateSurfaceError),
}

/// The main application state.
pub(crate) struct AppState {
    instance: wgpu::Instance,
    window: Window,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    size: winit::dpi::PhysicalSize<u32>,
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

        let size = window.get_window().inner_size();
        info!("Window size: {:?}", size);

        let surface = instance.create_surface(window.get_window_arc())?;
        info!("New wgpu surface successfully created");

        let capabilities = surface.get_capabilities(&adapter);
        info!("Surface capabilities: {:?}", capabilities);

        let format = capabilities.formats[0];
        info!("Surface format: {:?}", format);

        let mut state = Self {
            instance,
            window,
            adapter,
            device,
            queue,
            surface,
            format,
            size,
        };

        state.configure_surface();

        Ok(state)
    }

    pub(crate) fn run(&mut self) {
        self.render();
    }

    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    fn configure_surface(&mut self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            // Request compatibility with the sRGB-format texture view we're going to create later
            view_formats: vec![self.format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                unreachable!("No error scope registered, so validation errors will panic")
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self
                    .instance
                    .create_surface(self.window.get_window_arc().clone())
                    .unwrap();
                self.configure_surface();
                warn!("Surface texture was lost, but is now reconfigured");
                return;
            }
        };

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.get_window().pre_present_notify();
        self.queue.present(surface_texture);
    }
}
