use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::app::instance::InstanceRaw;
use crate::app::mesh::{CUBE_INDICES, CUBE_VERTICES, Vertex};
use crate::app::window::Window;
use crate::assets::AssetRegistry;
use crate::consts::app::{
    DESIRED_MAXIMUM_FRAME_LATENCY, PRESENT_MODE, SKIPPED_FRAME_DELAY, WINDOW_TITLE,
};
use crate::consts::overlay::{
    OVERLAY_FONT_SIZE, OVERLAY_FPS_SMOOTHING, OVERLAY_LINE_SPACING, OVERLAY_MARGIN,
};
use crate::consts::render::{
    CAMERA_FAR, CAMERA_FOV_Y_RADIANS, CAMERA_NEAR, CAMERA_POSITION, CUBE_SPIN_RADIANS_PER_SECOND,
    DEPTH_FORMAT,
};
use crate::consts::shader::ROOT_SHADER_PATH;
use crate::listeners::ListenerManager;
use crate::listeners::ListenerValue;
use crate::listeners::WindowListener;

use crate::routines::RoutineManager;
use crate::runtime::handle::{RuntimeCommand, RuntimeHandle};
use crate::runtime::routines::RuntimeRoutine;
use bevy_ecs::world::World;
use glam::camera::rh::{proj::directx::perspective, view::look_at_mat4};
use glam::{Mat4, Vec3};
use iced_wgpu::core::Renderer as _;
use iced_wgpu::core::text::Renderer as _;
use iced_wgpu::core::{Color, Font, Pixels, Point, Rectangle, Size, Text, alignment, text};
use iced_wgpu::graphics::{Shell, Viewport};
use rand::{Rng, RngExt};
use thiserror::Error;
use tracing::{error, info, warn};
use wgpu::naga::error::ShaderError;
use wgpu::util::DeviceExt;
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
    #[error("failed to load asset")]
    AssetError(#[from] crate::assets::AssetError),
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
    depth_view: wgpu::TextureView,
    routine_manager: RoutineManager,
    shader_modules: Vec<wgpu::ShaderModule>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    start_time: Instant,
    worlds: Vec<World>,
    current_world: usize,
    iced_renderer: iced_wgpu::Renderer,
    frame_count: u64,
    last_frame_time: Instant,
    fps: f32,
    asset_registry: AssetRegistry,
    listener_manager: ListenerManager,
}

impl AppState {
    /// Creates a new instance of [`AppState`].
    pub(crate) async fn new(event_loop: &ActiveEventLoop) -> Result<Self, AppStateError> {
        // create the window
        let window = Window::new(event_loop)?;
        window.get_window().set_title(WINDOW_TITLE);

        // create the wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
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
        let info = adapter.get_info();
        info!("Device info: {:?}", info);

        let size = window.get_window().inner_size();
        info!("Window size: {:?}", size);

        let surface = instance.create_surface(window.get_window_arc())?;
        info!("New wgpu surface successfully created");

        let capabilities = surface.get_capabilities(&adapter);
        info!("Surface capabilities: {:?}", capabilities);

        let format = capabilities.formats[0];
        info!("Surface format: {:?}", format);

        let depth_view = Self::create_depth_view(&device, size.width.max(1), size.height.max(1));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube vertex buffer"),
            contents: bytemuck::cast_slice(&CUBE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube index buffer"),
            contents: bytemuck::cast_slice(&CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instances = [InstanceRaw::from_model(Mat4::IDENTITY)];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let instance_count = instances.len() as u32;

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera uniform buffer"),
            contents: bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // the overlay draws to the same sRGB-suffixed view the cube pipeline
        // targets, so the iced engine's pipelines must be built for that format
        let iced_engine = iced_wgpu::Engine::new(
            &adapter,
            device.clone(),
            queue.clone(),
            format.add_srgb_suffix(),
            None,
            Shell::headless(),
        );
        let iced_renderer =
            iced_wgpu::Renderer::new(iced_engine, Font::DEFAULT, Pixels(OVERLAY_FONT_SIZE));
        info!("iced overlay renderer successfully created");

        let mut state = Self {
            instance,
            window,
            adapter,
            device,
            queue,
            surface,
            format,
            size,
            depth_view,
            routine_manager: RoutineManager::new(),
            shader_modules: Vec::new(),
            render_pipeline: None,
            vertex_buffer,
            index_buffer,
            index_count: CUBE_INDICES.len() as u32,
            instance_buffer,
            instance_count,
            camera_buffer,
            camera_bind_group,
            start_time: Instant::now(),
            worlds: Vec::from([World::new()]),
            current_world: 0,
            iced_renderer,
            frame_count: 0,
            last_frame_time: Instant::now(),
            fps: 0.0,
            asset_registry: AssetRegistry::new()?,
            listener_manager: ListenerManager::new(),
        };

        state.configure_surface();

        let root_source = include_str!("shaders/root.wgsl");
        state.compile_shader_str(root_source, Some("rootshader"));
        state.create_root_pipeline(&camera_bind_group_layout);

        Ok(state)
    }

    pub(crate) fn add_routine(&mut self, routine: Box<dyn RuntimeRoutine>) {
        info!("Adding new routine with name '{}'", routine.name());

        self.routine_manager.add(routine);
    }

    pub(crate) fn add_listener(&mut self, listener: ListenerValue) {
        info!("Adding new listener with name '{}'", listener.name());

        self.listener_manager.add_listener(listener);
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

    /// Creates a depth texture view sized to match the surface.
    fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    /// Builds the root render pipeline from the most recently compiled shader module.
    fn create_root_pipeline(&mut self, camera_bind_group_layout: &wgpu::BindGroupLayout) {
        let module = match self.shader_modules.last() {
            Some(module) => module,
            None => {
                error!("No shader module compiled");
                std::process::exit(1);
            }
        };

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("root pipeline layout"),
                bind_group_layouts: &[camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        };

        // one mat4x4<f32> model matrix per instance, split into 4 vec4 columns
        // since WGSL vertex attributes cap out at 4 components each
        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[[f32; 4]; 4]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                },
            ],
        };

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("root pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[vertex_buffer_layout, instance_buffer_layout],
                },
                fragment: Some(wgpu::FragmentState {
                    module,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    // must match the sRGB view format used in render()
                    targets: &[Some(self.format.add_srgb_suffix().into())],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        self.render_pipeline = Some(pipeline);
        info!("Root render pipeline successfully created");
    }

    /// Runs all enabled routines, returning the commands they issued this frame.
    fn run_routines(&mut self) -> Vec<RuntimeCommand> {
        let world = &mut self.worlds[self.current_world];
        let handle = RuntimeHandle::new(world);

        self.routine_manager.run(&handle);

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
        self.depth_view = Self::create_depth_view(&self.device, new_size.width, new_size.height);
    }

    fn configure_surface(&mut self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.format,
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

    #[inline]
    pub(crate) fn get_world(&self) -> &World {
        &self.worlds[self.current_world]
    }

    #[inline]
    pub(crate) fn get_world_mut(&mut self) -> &mut World {
        &mut self.worlds[self.current_world]
    }

    #[inline]
    pub(crate) fn get_current_world(&self) -> usize {
        self.current_world
    }

    #[inline]
    pub(crate) fn set_current_world(&mut self, world: usize) {
        self.current_world = world;
    }

    /// Recomputes the camera's view-projection matrix and each instance's
    /// model matrix for this frame, uploading both to their buffers.
    fn update_frame(&mut self) {
        let view = look_at_mat4(CAMERA_POSITION, Vec3::ZERO, Vec3::Y);
        let aspect = self.size.width.max(1) as f32 / self.size.height.max(1) as f32;
        let projection = perspective(CAMERA_FOV_Y_RADIANS, aspect, CAMERA_NEAR, CAMERA_FAR);
        let view_proj = projection * view;

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&view_proj.to_cols_array()),
        );

        let elapsed = self.start_time.elapsed().as_secs_f32();
        let model = Mat4::from_rotation_y(elapsed * CUBE_SPIN_RADIANS_PER_SECOND)
            * Mat4::from_rotation_x(elapsed * CUBE_SPIN_RADIANS_PER_SECOND * 0.6);
        let instances = [InstanceRaw::from_model(model)];
        self.queue
            .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
    }

    /// Updates the smoothed FPS estimate from the time elapsed since the
    /// previous frame.
    fn update_fps(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        if dt <= 0.0 {
            return;
        }

        let instantaneous = 1.0 / dt;
        self.fps = if self.frame_count <= 1 {
            instantaneous
        } else {
            self.fps + OVERLAY_FPS_SMOOTHING * (instantaneous - self.fps)
        };
    }

    /// Draws a single line of overlay text at the given logical position.
    fn draw_overlay_line(&mut self, content: String, position: Point, bounds: Size) {
        self.iced_renderer.fill_text(
            Text {
                content,
                bounds,
                size: Pixels(OVERLAY_FONT_SIZE),
                line_height: text::LineHeight::default(),
                font: Font::MONOSPACE,
                align_x: text::Alignment::default(),
                align_y: alignment::Vertical::Top,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
            },
            position,
            Color::WHITE,
            Rectangle::new(position, bounds),
        );
    }

    /// Draws the debug overlay (frame counter + FPS) on top of whatever is
    /// already in `texture_view`, via its own iced_wgpu submission.
    fn draw_overlay(&mut self, texture_view: &wgpu::TextureView) {
        self.update_fps();

        let scale_factor = self.window.get_window().scale_factor() as f32;
        let viewport = Viewport::with_physical_size(
            Size::new(self.size.width.max(1), self.size.height.max(1)),
            scale_factor,
        );

        let logical_size = viewport.logical_size();
        self.iced_renderer.reset(Rectangle::with_size(logical_size));

        // the text's own `bounds` is the box cosmic-text lays it out within
        // (not just a clip rect) — an infinite bound breaks that layout math
        // and only the first glyph ends up drawn, so give it the actual
        // available space instead.
        let text_bounds = Size::new(
            (logical_size.width - 2.0 * OVERLAY_MARGIN).max(0.0),
            (logical_size.height - 2.0 * OVERLAY_MARGIN).max(0.0),
        );

        self.draw_overlay_line(
            format!("frame {}", self.frame_count),
            Point::new(OVERLAY_MARGIN, OVERLAY_MARGIN),
            text_bounds,
        );
        self.draw_overlay_line(
            format!("{:.0} fps", self.fps),
            Point::new(OVERLAY_MARGIN, OVERLAY_MARGIN + OVERLAY_LINE_SPACING),
            text_bounds,
        );

        // clear_color: None so this submission loads (rather than clears) the
        // cube pass's output; present() submits its own encoder to the same
        // wgpu::Queue, so it's guaranteed to execute after the cube's submit below.
        self.iced_renderer
            .present(None, self.format.add_srgb_suffix(), texture_view, &viewport);
    }

    fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(texture) if texture.suboptimal => {
                drop(texture);
                self.configure_surface();
                None
            }
            Ok(texture) => Some(texture),
            Err(wgpu::SurfaceError::Timeout) => None,
            Err(wgpu::SurfaceError::Outdated) => {
                self.configure_surface();
                None
            }
            Err(wgpu::SurfaceError::Lost) => {
                self.surface = self
                    .instance
                    .create_surface(self.window.get_window_arc().clone())
                    .unwrap();
                self.configure_surface();
                warn!("Surface texture was lost, but is now reconfigured");
                None
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("Out of memory while acquiring surface texture");
                std::process::exit(1);
            }
            Err(wgpu::SurfaceError::Other) => {
                warn!("Failed to acquire surface texture with a generic error");
                None
            }
        };

        let Some(surface_texture) = surface_texture else {
            // vsync only paces frames that present; without this, skipped frames
            // (occluded/outdated surface) spin the redraw loop at 100% CPU
            std::thread::sleep(SKIPPED_FRAME_DELAY);
            return;
        };

        self.update_frame();

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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Draw the cube with the root pipeline.
        if let Some(pipeline) = &self.render_pipeline {
            renderpass.set_pipeline(pipeline);
            renderpass.set_bind_group(0, &self.camera_bind_group, &[]);
            renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            renderpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            renderpass.draw_indexed(0..self.index_count, 0, 0..self.instance_count);
        }

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.queue.submit([encoder.finish()]);

        self.frame_count += 1;
        self.draw_overlay(&texture_view);

        self.window.get_window().pre_present_notify();
        surface_texture.present();
    }
}
