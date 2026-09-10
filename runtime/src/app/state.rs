use std::path::{Path, PathBuf};

use crate::app::window::Window;
use crate::consts::app::{
    DESIRED_MAXIMUM_FRAME_LATENCY, PRESENT_MODE, SKIPPED_FRAME_DELAY, WINDOW_TITLE,
};
use crate::consts::shader::ROOT_SHADER_PATH;
use crate::runtime::handle::{RuntimeCommand, RuntimeHandle};
use crate::runtime::routines::RuntimeRoutine;
use rand::{Rng, RngExt};
use thiserror::Error;
use tracing::{error, info, warn};
use wgpu::naga::error::ShaderError;
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
    #[error("failed to load shader")]
    ShaderError,
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
    routines: Vec<Box<dyn RuntimeRoutine>>,
    shader_modules: Vec<wgpu::ShaderModule>,
    render_pipeline: Option<wgpu::RenderPipeline>,
}

impl AppState {
    /// Creates a new instance of [`AppState`].
    pub(crate) async fn new(event_loop: &ActiveEventLoop) -> Result<Self, AppStateError> {
        // create the window
        let window = Window::new(event_loop)?;
        window.get_window().set_title(WINDOW_TITLE);

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
            routines: Vec::new(),
            shader_modules: Vec::new(),
            render_pipeline: None,
        };

        state.configure_surface();

        let root_source = include_str!("../../../runtime/assets/shaders/root.wgsl");
        state.compile_shader_str(root_source, Some("rootshader"));
        state.create_root_pipeline();

        Ok(state)
    }

    pub(crate) fn add_routine(&mut self, routine: Box<dyn RuntimeRoutine>) {
        info!("Adding new routine with name '{}'", routine.name());

        self.routines.push(routine);
        self.routines.sort_by_key(|r| r.priority());
    }

    pub(crate) fn compile_shader_str(&mut self, shader_source: &str, name: Option<&str>) {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: name,
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        self.shader_modules.push(shader);

        info!("Compiled shader {:?}", name);
    }

    /// Builds the root render pipeline from the most recently compiled shader module.
    fn create_root_pipeline(&mut self) {
        let module = match self.shader_modules.last() {
            Some(module) => module,
            None => {
                error!("No shader module compiled");
                std::process::exit(1);
            }
        };

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("root pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    // must match the sRGB view format used in render()
                    targets: &[Some(self.format.add_srgb_suffix().into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        self.render_pipeline = Some(pipeline);
        info!("Root render pipeline successfully created");
    }

    /// Runs all enabled routines, returning the commands they issued this frame.
    fn run_routines(&mut self) -> Vec<RuntimeCommand> {
        let handle = RuntimeHandle::new();

        for routine in &mut self.routines {
            if !routine.is_enabled() {
                continue;
            }

            routine.frame(&handle);
        }

        handle.take_commands()
    }

    pub(crate) fn run(&mut self) -> Vec<RuntimeCommand> {
        self.render();

        self.run_routines()
    }

    #[inline]
    pub(crate) fn get_window(&self) -> &Window {
        &self.window
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // a zero-sized surface is invalid to configure (e.g. mid-transition or minimized)
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

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
            desired_maximum_frame_latency: DESIRED_MAXIMUM_FRAME_LATENCY,
            present_mode: PRESENT_MODE,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => Some(texture),
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => None,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                None
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                None
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
                None
            }
        };

        let Some(surface_texture) = surface_texture else {
            // vsync only paces frames that present; without this, skipped frames
            // (occluded/outdated surface) spin the redraw loop at 100% CPU
            std::thread::sleep(SKIPPED_FRAME_DELAY);
            return;
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
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // Draw with the root pipeline.
        if let Some(pipeline) = &self.render_pipeline {
            renderpass.set_pipeline(pipeline);
            renderpass.draw(0..3, 0..1);
        }

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);
        self.window.get_window().pre_present_notify();
        self.queue.present(surface_texture);
    }
}
