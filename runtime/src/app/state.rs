use crate::app::instance::InstanceCache;
use crate::app::mesh::GpuMeshCache;
use crate::app::window::Window;
use crate::assets::{AssetRegistry, Vertex};
use crate::consts::app::{
    DESIRED_MAXIMUM_FRAME_LATENCY, PRESENT_MODE, SKIPPED_FRAME_DELAY, WINDOW_TITLE,
};
use crate::consts::overlay::{OVERLAY_FONT_SIZE, OVERLAY_FPS_SAMPLE_INTERVAL, OVERLAY_MARGIN};
use crate::consts::render::{
    CAMERA_FAR, CAMERA_FOV_Y_RADIANS, CAMERA_NEAR, CAMERA_POSITION, DEPTH_FORMAT,
};
use crate::gpu::{BufferType, GpuBindGroup, GpuBindGroupBuilder, GpuBuffer, GpuResources};
use crate::listeners::ListenerManager;
use crate::listeners::ListenerValue;
use crate::listeners::MouseInput;
use crate::listeners::WindowListener;
use crate::listeners::{InputListener, ScreenPosition};
use crate::misc::data::format_bytes;
use crate::{BUILTIN_FRUSTUM_SHADER_PATH, BUILTIN_ROOT_SHADER_PATH, ShaderAsset};
use std::path::Path;
use std::time::Instant;

use crate::routines::RoutineManager;
use crate::runtime::handle::{RuntimeCommand, RuntimeHandle};
use crate::runtime::routines::RuntimeRoutine;
use bevy_ecs::world::World;
use glam::camera::rh::{proj::directx::perspective, view::look_at_mat4};
use glam::{Mat4, Vec3};
use iced_runtime::user_interface::{Cache, UserInterface};
use iced_wgpu::core::renderer::Style as RendererStyle;
use iced_wgpu::core::{Color, Element, Font, Pixels, Size, mouse};
use iced_wgpu::graphics::{Shell, Viewport};
use iced_widget::{container, table, text};
use thiserror::Error;
use tracing::{error, info, warn};
use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;

#[derive(Error, Debug)]
pub(crate) enum AppStateError {
    #[error("failed to create window: {0}")]
    WindowError(#[from] crate::app::window::WindowError),
    #[error("failed to create wgpu adapter: {0}")]
    WgpuAdapterError(#[from] wgpu::RequestAdapterError),
    #[error("failed to create wgpu device: {0}")]
    WgpuDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("failed to create wgpu surface: {0}")]
    WgpuSurfaceError(#[from] wgpu::CreateSurfaceError),
    #[error("failed to load shader")]
    ShaderError,
    #[error("failed to load asset: {0}")]
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
    gpu_meshes: GpuMeshCache,
    instances: InstanceCache,
    start_time: Instant,
    worlds: Vec<World>,
    current_world: usize,
    iced_renderer: iced_wgpu::Renderer,
    frame_count: u64,
    fps_window_start: Instant,
    fps_window_frames: u32,
    fps: f32,
    asset_registry: AssetRegistry,
    listener_manager: ListenerManager,
    mesh_count: usize,
    gpu_resources: GpuResources,
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

        let mut gpu_resources = GpuResources::new();

        let camera_buffer = GpuBuffer::builder()
            .name("camera buffer")
            .device(&device)
            .number(0)
            .contents(bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()))
            .visibility(wgpu::ShaderStages::VERTEX_FRAGMENT)
            .ty(BufferType::Uniform)
            .build();

        let frustum_buffer = GpuBuffer::builder()
            .name("frustum buffer")
            .device(&device)
            .number(1)
            .visibility(wgpu::ShaderStages::COMPUTE)
            .contents(&[0])
            .ty(BufferType::Uniform)
            .build();

        let group0 = GpuBindGroup::builder()
            .name("camera group")
            .device(&device)
            .number(0)
            .buffer(camera_buffer)
            .buffer(frustum_buffer)
            .build();

        // --- GROUP 0 END --- //

        let objects_array_buffer = GpuBuffer::builder()
            .name("objects array buffer")
            .device(&device)
            .number(0)
            .contents(&[0])
            .visibility(wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE)
            .ty(BufferType::BufferReadOnly)
            .build();

        let visible_objects_array_buffer = GpuBuffer::builder()
            .name("visible objects array buffer")
            .device(&device)
            .number(1)
            .contents(&[0])
            .visibility(wgpu::ShaderStages::COMPUTE)
            .ty(BufferType::BufferReadable)
            .build();

        let draw_args_buffer = GpuBuffer::builder()
            .name("draw args buffer")
            .device(&device)
            .number(2)
            .contents(&[0])
            .visibility(wgpu::ShaderStages::COMPUTE)
            .ty(BufferType::BufferReadable)
            .usage(wgpu::BufferUsages::INDIRECT)
            .build();

        let group1 = GpuBindGroup::builder()
            .name("objects group")
            .device(&device)
            .number(1)
            .buffer(objects_array_buffer)
            .buffer(visible_objects_array_buffer)
            .buffer(draw_args_buffer)
            .build();

        // --- GROUP 1 END --- //

        gpu_resources.add_group(group0);
        gpu_resources.add_group(group1);

        let depth_view = Self::create_depth_view(&device, size.width.max(1), size.height.max(1));

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
            gpu_meshes: GpuMeshCache::new(),
            instances: InstanceCache::new(),
            start_time: Instant::now(),
            worlds: Vec::from([World::new()]),
            current_world: 0,
            iced_renderer,
            frame_count: 0,
            fps_window_start: Instant::now(),
            fps_window_frames: 0,
            fps: 0.0,
            asset_registry: AssetRegistry::new()?,
            listener_manager: ListenerManager::new(),
            mesh_count: 0,
            gpu_resources,
        };

        state.configure_surface();

        let root_shader = state
            .asset_registry
            .read_shader_from_path(Path::new(BUILTIN_ROOT_SHADER_PATH), "bi.rootshader")?;
        state.compile_shader_asset(root_shader, Some("rootshader"));

        let cull_shader = state
            .asset_registry
            .read_shader_from_path(Path::new(BUILTIN_FRUSTUM_SHADER_PATH), "bi.frustumshader")?;
        state.compile_shader_asset(cull_shader, Some("cullshader"));

        state.create_root_pipeline();

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

    pub(crate) fn notify_window_created(&mut self) {
        let world = &mut self.worlds[self.current_world];
        let handle = RuntimeHandle::new(world);
        self.listener_manager.on_window_create(&handle);
    }

    pub(crate) fn notify_window_close(&mut self) {
        let world = &mut self.worlds[self.current_world];
        let handle = RuntimeHandle::new(world);
        self.listener_manager.on_window_close(&handle);
    }

    pub(crate) fn notify_window_focus(&mut self) {
        let world = &mut self.worlds[self.current_world];
        let handle = RuntimeHandle::new(world);
        self.listener_manager.on_window_focus(&handle);
    }

    pub(crate) fn notify_cursor_moved(&mut self, pos: ScreenPosition) {
        let world = &mut self.worlds[self.current_world];
        let handle = RuntimeHandle::new(world);
        self.listener_manager
            .on_mouse_input(MouseInput::Move(pos), &handle);
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

    pub(crate) fn compile_shader_asset(&mut self, shader_asset: ShaderAsset, name: Option<&str>) {
        let shader_source = shader_asset.get_data(&self.asset_registry).to_string();
        self.compile_shader_str(&shader_source, name);
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
    fn create_root_pipeline(&mut self) {
        let module = match self.shader_modules.first() {
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
                bind_group_layouts: &self.gpu_resources.get_group_layouts(),
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
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
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

        let world = &mut self.worlds[self.current_world];
        let handle = RuntimeHandle::new(world);
        self.listener_manager
            .on_window_resize(new_size.width, new_size.height, &handle);
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

    /// Recomputes the camera's view-projection matrix and uploads it, then
    /// uploads this frame's mesh instances.
    fn update_frame(&mut self) {
        let view = look_at_mat4(CAMERA_POSITION, Vec3::ZERO, Vec3::Y);
        let aspect = self.size.width.max(1) as f32 / self.size.height.max(1) as f32;
        let projection = perspective(CAMERA_FOV_Y_RADIANS, aspect, CAMERA_NEAR, CAMERA_FAR);
        let view_proj = projection * view;

        if let Some(camera_buffer) = self
            .gpu_resources
            .get_groups()
            .first()
            .and_then(|group| group.get_buffers().first())
        {
            self.queue.write_buffer(
                camera_buffer.get_buffer(),
                0,
                bytemuck::cast_slice(&view_proj.to_cols_array()),
            );
        }

        self.upload_instances();
    }

    /// Syncs changed `(Mesh, Transform)` entities into the GPU instance
    /// buffers, and uploads any mesh geometry drawn for the first time.
    fn upload_instances(&mut self) {
        let world = &mut self.worlds[self.current_world];
        self.instances
            .sync(&self.device, &self.queue, self.current_world, world);
        self.mesh_count = self.instances.len();

        for (asset_id, _, _) in self.instances.batches() {
            self.gpu_meshes
                .ensure_uploaded(&self.device, &self.asset_registry, asset_id);
        }
    }

    /// Counts this frame and, once per [`OVERLAY_FPS_SAMPLE_INTERVAL`],
    /// refreshes the FPS readout as frames counted / time elapsed.
    ///
    /// Averaging per-frame `1 / dt` instead would be skewed by any
    /// near-instant frames (e.g. when the swapchain hands back an already-free
    /// drawable), reporting hundreds of FPS while actually presenting a few.
    fn update_fps(&mut self) {
        self.fps_window_frames += 1;

        let elapsed = self.fps_window_start.elapsed();
        if elapsed < OVERLAY_FPS_SAMPLE_INTERVAL {
            return;
        }

        self.fps = self.fps_window_frames as f32 / elapsed.as_secs_f32();
        self.fps_window_frames = 0;
        self.fps_window_start = Instant::now();
    }

    /// A single label/value row of the debug overlay's stat table.
    fn overlay_rows(&self) -> Vec<(&'static str, String)> {
        vec![
            ("frame", self.frame_count.to_string()),
            ("fps", format!("{:.0}", self.fps)),
            ("meshes", self.mesh_count.to_string()),
            (
                "rw asst id",
                self.asset_registry.current_raw_asset_id().to_string(),
            ),
            (
                "msh asst id",
                self.asset_registry.current_mesh_asset_id().to_string(),
            ),
            (
                "shdr asst id",
                self.asset_registry.current_shader_asset_id().to_string(),
            ),
            (
                "asst reg sz",
                format_bytes(self.asset_registry.total_bytes()),
            ),
        ]
    }

    /// Builds the debug overlay's widget tree: a two-column stat/value table
    /// wrapped in a margin. Adding a new stat only means pushing a row in
    /// [`Self::overlay_rows`] — iced's layout engine positions everything.
    fn make_overlay_element<'a>(
        rows: Vec<(&'static str, String)>,
    ) -> Element<'a, (), iced_wgpu::core::Theme, iced_wgpu::Renderer> {
        let cell_text = |content: String| {
            text(content)
                .font(Font::MONOSPACE)
                .size(Pixels(OVERLAY_FONT_SIZE))
                .color(Color::WHITE)
        };

        let stat_column = table::column(
            cell_text("stat".to_string()),
            move |(label, _): (&'static str, String)| cell_text(label.to_string()),
        );
        let value_column = table::column(
            cell_text("value".to_string()),
            move |(_, value): (&'static str, String)| cell_text(value),
        );

        let stat_table = table::table([stat_column, value_column], rows)
            .padding_x(6.0)
            .padding_y(2.0)
            // removes table borders
            .separator(0.0);

        container(stat_table).padding(OVERLAY_MARGIN).into()
    }

    /// Draws the debug overlay (frame counter + FPS + asset ids) on top of
    /// whatever is already in `texture_view`, via its own iced_wgpu submission.
    fn draw_overlay(&mut self, texture_view: &wgpu::TextureView) {
        self.update_fps();

        let scale_factor = self.window.get_window().scale_factor() as f32;
        let viewport = Viewport::with_physical_size(
            Size::new(self.size.width.max(1), self.size.height.max(1)),
            scale_factor,
        );

        let overlay_element = Self::make_overlay_element(self.overlay_rows());
        let mut user_interface = UserInterface::build(
            overlay_element,
            viewport.logical_size(),
            Cache::new(),
            &mut self.iced_renderer,
        );
        user_interface.draw(
            &mut self.iced_renderer,
            &iced_wgpu::core::Theme::Dark,
            &RendererStyle::default(),
            mouse::Cursor::Unavailable,
        );
        // clear_color: None so this submission loads (rather than clears) the
        // cube pass's output; present() submits its own encoder to the same
        // wgpu::Queue, so it's guaranteed to execute after the cube's submit below.
        self.iced_renderer
            .present(None, self.format.add_srgb_suffix(), texture_view, &viewport);

        // present() only weak-references each Text widget's shaped paragraph, so
        // user_interface (which holds the strong ref in its widget::Tree) must
        // outlive it, or glyphs silently fail to upgrade and never draw.
        drop(user_interface);
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

        // Draw every mesh with the root pipeline, one instanced call per mesh.
        if let Some(pipeline) = &self.render_pipeline {
            renderpass.set_pipeline(pipeline);
            for (index, group) in self.gpu_resources.get_groups().iter().enumerate() {
                renderpass.set_bind_group(index as u32, group.get_group(), &[]);
            }
            for (asset_id, instance_buffer, count) in self.instances.batches() {
                if let Some(mesh) = self.gpu_meshes.get(asset_id) {
                    renderpass.set_vertex_buffer(1, instance_buffer.slice(..));
                    mesh.draw(&mut renderpass, 0..count);
                }
            }
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
