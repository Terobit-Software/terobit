use chrono::Utc;
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};

use glam::{Mat4, Vec2, Vec3};
use wgpu::{
    BufferSlice, Device, FrontFace, MultisampleState, PipelineCompilationOptions, PrimitiveState,
    PrimitiveTopology, Texture, TextureView, util::DeviceExt, wgc::device,
};
use winit::{event_loop::OwnedDisplayHandle, window::Window};

use crate::{
    assets::AssetReader,
    consts::{
        DEFAULT_CAMERA_Z_CLOSE, DEFAULT_CAMERA_Z_FAR, DEFAULT_FOV, MAX_HUD_GLYPHS,
        MSAA_SAMPLE_COUNT,
    },
    ecs::{Camera, PhysicsScene, Transform},
    profiling::Profiler,
    rendering::{
        FontAtlas, GLYPH_H, GLYPH_W, GlyphInstance, GpuMesh, InstanceRaw, MeshSourceHandler,
        MeshSourceId, QuadVertex, RenderCamera, Vertex, VertexRaw, VertexTrait, accumulate_meshes,
    },
};
use bevy_ecs::world::World;
use bevy_ecs::{prelude::*, world::WorldId};

/// everything `init_text_rendering` builds, bundled up so it doesn't have to return
/// an 8-tuple
struct TextRenderResources {
    screen_buffer: wgpu::Buffer,
    screen_bind_group: wgpu::BindGroup,
    atlas_bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    quad_vertex_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    glyph_instance_buffer: wgpu::Buffer,
    glyph_uv: HashMap<char, [f32; 4]>,
}

/// owns all wgpu objects and scene state. one instance lives inside `App` after `resumed` fires
pub(crate) struct AppState {
    /// the wgpu instance of the app
    instance: wgpu::Instance,
    /// the window for the app
    window: Arc<Window>,
    /// the surface to be rendered onto
    surface: wgpu::Surface<'static>,
    /// the gpu basically
    device: wgpu::Device,
    /// the queue to speak with mr gpu
    queue: wgpu::Queue,
    /// the size of the surface
    size: winit::dpi::PhysicalSize<u32>,
    /// the format of the surface, idk
    surface_format: wgpu::TextureFormat,
    /// dis is how we render
    pipeline: wgpu::RenderPipeline,

    /// the camera buffer inside the camera_bind_group
    camera_buffer: wgpu::Buffer,
    /// the camera resource group for the gpu
    camera_bind_group: wgpu::BindGroup,

    /// the screen depth texture used for 3d rendering
    depth_texture: wgpu::Texture,
    /// the depth view, pointing to the depth_texture
    depth_view: wgpu::TextureView,

    /// the current camera
    camera: RenderCamera,
    /// how many frames have been since last_time was updated
    frame_count: i32,
    /// last time the framerate was updated
    last_time: i64,

    /// the currently being rendered scene
    current_scene: Option<String>,
    /// the scenes available
    scenes: HashMap<String, World>,

    mesh_registry: HashMap<MeshSourceId, GpuMesh>,
    instance_buffer: wgpu::Buffer,
    /// counter for assigning unique ids to custom meshes. starts at 1 (0 and 1 are
    /// reserved for the built-in cube and plane), so the first custom mesh gets id 2
    mesh_id_counter: usize,
    /// cache mapping file path to its registered MeshSourceId so we don't re-upload the same file
    mesh_path_cache: HashMap<PathBuf, MeshSourceId>,
    /// cpu-side vertex positions per mesh, kept around so `Collider::inherit()` can
    /// build a physics shape from any mesh (built-in or asset-loaded)
    mesh_points: HashMap<MeshSourceId, Vec<Vec3>>,
    /// clock used to compute real per-frame dt, drives both `on_frame` and the physics step
    physics_clock: std::time::Instant,
    /// the physics simulation for the current scene
    physics: PhysicsScene,

    /// labeled timings/counters for engine internals, exposed to game code too via
    /// `RuntimeContext`. toggled on screen with f7+p
    profiler: Profiler,
    show_profiler_hud: bool,
    /// screen_size uniform bind group for the text pipeline, separate from the 3d
    /// camera's since text is drawn in raw pixel space, not world space
    text_screen_buffer: wgpu::Buffer,
    text_screen_bind_group: wgpu::BindGroup,
    /// font atlas bind group (texture + sampler), built once, never changes
    text_atlas_bind_group: wgpu::BindGroup,
    text_pipeline: wgpu::RenderPipeline,
    text_quad_vertex_buffer: wgpu::Buffer,
    text_quad_index_buffer: wgpu::Buffer,
    /// rewritten every frame the hud is visible
    glyph_instance_buffer: wgpu::Buffer,
    /// character -> uv rect in the atlas, kept around to lay out hud text each frame
    font_glyph_uv: HashMap<char, [f32; 4]>,
}

impl AppState {
    /// create a new app state
    pub async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::debugging(),
            ..wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display))
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        println!("{:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let shader_source = include_str!("shaders/root.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("root shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let camera = RenderCamera::new(Vec3::new(2.0, 2.0, 3.0), Vec3::ZERO);

        let mvp = camera.build_view_proj(size.width, size.height);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::cast_slice(&[mvp]),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
            ..Default::default()
        });

        let (depth_texture, depth_view) =
            Self::create_depth_texture(&device, size, MSAA_SAMPLE_COUNT);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            label: None,
            layout: Some(&pipeline_layout),
            primitive: PrimitiveState {
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            cache: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                alpha_to_coverage_enabled: false,
                count: MSAA_SAMPLE_COUNT,
                mask: 0xFFFFFFFF,
            },
            multiview_mask: None,
        });

        // pre-allocate space for up to 500,001 instances
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 500000) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let text = Self::init_text_rendering(&device, &queue, surface_format, size);

        let mut state = Self {
            instance,
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            pipeline,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            depth_view,
            camera,
            frame_count: 0,
            last_time: Utc::now().timestamp_millis(),
            scenes: HashMap::new(),
            current_scene: None,
            instance_buffer,
            mesh_registry: HashMap::new(),
            mesh_id_counter: 1,
            mesh_path_cache: HashMap::new(),
            mesh_points: HashMap::new(),
            physics_clock: std::time::Instant::now(),
            physics: PhysicsScene::new(),
            profiler: Profiler::new(),
            show_profiler_hud: false,
            text_screen_buffer: text.screen_buffer,
            text_screen_bind_group: text.screen_bind_group,
            text_atlas_bind_group: text.atlas_bind_group,
            text_pipeline: text.pipeline,
            text_quad_vertex_buffer: text.quad_vertex_buffer,
            text_quad_index_buffer: text.quad_index_buffer,
            glyph_instance_buffer: text.glyph_instance_buffer,
            font_glyph_uv: text.glyph_uv,
        };

        let res = MeshSourceHandler::init_default_meshes(&mut state);
        if res.is_err() {
            panic!("failed to upload default meshes");
        } else {
            println!("uploaded default meshes");
        }

        state.upload_camera();
        state.configure_surface();

        state
    }

    /// builds everything the profiler hud's text needs: the font atlas texture, the
    /// screen-space pipeline that draws glyph quads, and their backing buffers.
    /// separate from the 3d pipeline since text lives in pixel space, not world space
    fn init_text_rendering(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        size: winit::dpi::PhysicalSize<u32>,
    ) -> TextRenderResources {
        let screen_size = [size.width as f32, size.height as f32];
        let text_screen_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("text screen size buffer"),
            contents: bytemuck::cast_slice(&screen_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("text screen bind group layout"),
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

        let text_screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text screen bind group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: text_screen_buffer.as_entire_binding(),
            }],
        });

        let atlas = FontAtlas::build();
        let atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font atlas texture"),
            size: wgpu::Extent3d { width: atlas.width, height: atlas.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &atlas_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &atlas.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas.width),
                rows_per_image: Some(atlas.height),
            },
            wgpu::Extent3d { width: atlas.width, height: atlas.height, depth_or_array_layers: 1 },
        );

        let atlas_view = atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("font atlas bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let text_atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("font atlas bind group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&atlas_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&atlas_sampler) },
            ],
        });

        let text_shader_source = include_str!("shaders/text.wgsl");
        let text_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text shader"),
            source: wgpu::ShaderSource::Wgsl(text_shader_source.into()),
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[Some(&screen_bind_group_layout), Some(&atlas_bind_group_layout)],
            ..Default::default()
        });

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &text_shader_module,
                entry_point: Some("vs_main"),
                buffers: &[QuadVertex::desc(), GlyphInstance::desc()],
                compilation_options: PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
            },
            fragment: Some(wgpu::FragmentState {
                module: &text_shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            // text is a screen-space overlay, drawn on top of everything regardless of
            // 3d depth, but the pipeline still needs a depth_stencil state that's
            // compatible with the render pass's depth attachment
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                alpha_to_coverage_enabled: false,
                count: MSAA_SAMPLE_COUNT,
                mask: 0xFFFFFFFF,
            },
            cache: None,
            multiview_mask: None,
        });

        let text_quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("text quad vertex buffer"),
            contents: bytemuck::cast_slice(&QuadVertex::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let text_quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("text quad index buffer"),
            contents: bytemuck::cast_slice(&QuadVertex::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let glyph_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glyph instance buffer"),
            size: std::mem::size_of::<GlyphInstance>() as u64 * MAX_HUD_GLYPHS,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        TextRenderResources {
            screen_buffer: text_screen_buffer,
            screen_bind_group: text_screen_bind_group,
            atlas_bind_group: text_atlas_bind_group,
            pipeline: text_pipeline,
            quad_vertex_buffer: text_quad_vertex_buffer,
            quad_index_buffer: text_quad_index_buffer,
            glyph_instance_buffer,
            glyph_uv: atlas.glyph_uv,
        }
    }

    // pulls position and rotation off the first camera entity in the scene, if any
    fn upload_camera(&mut self) {
        let camera_data: Option<(Vec3, Vec3)> = if let Some(scene_name) = self.current_scene.clone() {
            if let Some(scene) = self.get_scene_mut(scene_name) {
                scene
                    .query::<(&Transform, &Camera)>()
                    .iter(scene)
                    .next()
                    .map(|(transform, _)| {
                        // forward direction: default facing is +x, rotated by transform.rotation
                        let forward = transform.rotation * Vec3::X;
                        (transform.position, transform.position + forward)
                    })
            } else {
                None
            }
        } else {
            None
        };

        if let Some((pos, target)) = camera_data {
            self.camera.position = pos;
            self.camera.target = target;
        }

        let mvp = self
            .camera
            .build_view_proj(self.size.width, self.size.height);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[mvp]));
    }

    /// uploads vertex and index data for a mesh to the gpu and registers it by id.
    /// also keeps a cpu-side copy of the vertex positions so `Collider::inherit()`
    /// can build a physics shape from this mesh later
    pub fn upload_mesh(&mut self, id: MeshSourceId, vertices: &[VertexRaw], indices: &[u32]) {
        let points = vertices
            .iter()
            .map(|v| Vec3::new(v.position[0], v.position[1], v.position[2]))
            .collect();
        self.mesh_points.insert(id, points);

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        self.mesh_registry.insert(
            id,
            GpuMesh {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            },
        );
    }

    /// (re)configures the wgpu surface with the current size and format. call after resize
    pub fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::Fifo,
        };
        self.surface.configure(&self.device, &surface_config);
    }

    /// returns a reference to the underlying winit window
    pub fn get_window(&self) -> &Window {
        &self.window
    }

    /// seconds elapsed since the last call, resets the clock. drives `on_frame`'s delta
    /// and the physics step so both run at the actual frame rate instead of a fixed 0.0
    pub(crate) fn tick_delta(&mut self) -> f32 {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.physics_clock).as_secs_f32();
        self.physics_clock = now;
        dt
    }

    /// steps physics (gravity, integration, contact resolution) for the current scene
    pub(crate) fn step_physics(&mut self, dt: f32) {
        let Some(scene_name) = self.current_scene.as_ref() else { return };
        let Some(scene) = self.scenes.get_mut(scene_name) else { return };
        self.physics.step(scene, dt, &self.mesh_points, &mut self.profiler);
    }

    /// engine-wide profiler, also reachable from game code through `RuntimeContext`
    pub(crate) fn profiler_mut(&mut self) -> &mut Profiler {
        &mut self.profiler
    }

    /// flips whether the f7+p profiler hud draws over the scene
    pub(crate) fn toggle_profiler_hud(&mut self) {
        self.show_profiler_hud = !self.show_profiler_hud;
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        new_size: winit::dpi::PhysicalSize<u32>,
        sample_count: u32,
    ) -> (Texture, TextureView) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: new_size.width,
                height: new_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count, // ← must match your MSAA sample count (e.g. 4)
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        (depth_texture, depth_view)
    }

    /// handles window resize: updates size, reconfigures surface, recreates depth texture
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
        (self.depth_texture, self.depth_view) =
            Self::create_depth_texture(&self.device, new_size, MSAA_SAMPLE_COUNT);

        self.upload_camera();

        let screen_size = [new_size.width as f32, new_size.height as f32];
        self.queue.write_buffer(&self.text_screen_buffer, 0, bytemuck::cast_slice(&screen_size));
    }

    /// lays out one line of hud text into glyph quads, top-left anchored at (x, y)
    /// in pixel space. characters missing from the font (shouldn't happen for the
    /// uppercase/digit/symbol text the hud prints) are skipped, never a panic
    fn append_line_glyphs(
        &self,
        text: &str,
        x: f32,
        y: f32,
        scale: f32,
        color: [f32; 4],
        out: &mut Vec<GlyphInstance>,
    ) {
        let advance = (GLYPH_W as f32 + 1.0) * scale;
        let mut cursor_x = x;

        for ch in text.chars() {
            if let Some(uv) = self.font_glyph_uv.get(&ch) {
                out.push(GlyphInstance {
                    screen_pos: [cursor_x, y],
                    size: [GLYPH_W as f32 * scale, GLYPH_H as f32 * scale],
                    uv_min: [uv[0], uv[1]],
                    uv_max: [uv[2], uv[3]],
                    color,
                });
            }
            cursor_x += advance;
        }
    }

    /// builds the f7+p profiler hud: one line per profiled label with its last/avg
    /// timing, plus any extra values attached to it (instance counts, draw calls, etc)
    fn build_hud_glyphs(&self) -> Vec<GlyphInstance> {
        const SCALE: f32 = 2.0;
        const COLOR: [f32; 4] = [0.2, 1.0, 0.4, 1.0];
        const MARGIN: f32 = 10.0;
        let line_height = (GLYPH_H as f32 + 2.0) * SCALE;

        let mut instances = Vec::new();
        let mut y = MARGIN;

        for (label, entry) in self.profiler.entries() {
            let mut line =
                format!("{}: {:.2}MS AVG:{:.2}MS", label.to_uppercase(), entry.last_ms, entry.avg_ms);

            let mut extra: Vec<_> = entry.values.iter().collect();
            extra.sort_by_key(|(key, _)| key.to_string());
            for (key, value) in extra {
                line.push_str(&format!(" {}:{:.0}", key.to_uppercase(), value));
            }

            self.append_line_glyphs(&line, MARGIN, y, SCALE, COLOR, &mut instances);
            y += line_height;
        }

        instances.truncate(MAX_HUD_GLYPHS as usize);
        instances
    }

    /// renders the current frame: uploads instance data, submits draw calls, presents
    pub fn render(&mut self) {
        let surface_texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => {
                self.window.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                drop(texture);
                self.configure_surface();
                self.window.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                self.window.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                self.surface = self.instance.create_surface(self.window.clone()).unwrap();
                self.configure_surface();
                self.window.request_redraw();
                return;
            }
            wgpu::CurrentSurfaceTexture::Validation => unreachable!(),
        };

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // draw calls and instance uploads happen before the renderpass starts
        let accumulate_start = std::time::Instant::now();
        let draw_calls: Vec<(MeshSourceId, usize, u64, u64)> =
            if let Some(scene_name) = self.current_scene.clone() {
                if let Some(scene) = self.get_scene_mut(scene_name) {
                    let containers = accumulate_meshes(scene);
                    let mut current_offset: u64 = 0;
                    let mut calls = vec![];

                    for (mesh_id, container) in &containers {
                        let instances: Vec<InstanceRaw> = container
                            .transforms
                            .iter()
                            .zip(container.colors.iter())
                            .map(|(t, &color)| InstanceRaw::new(*t, color))
                            .collect();
                        let data: &[u8] = bytemuck::cast_slice(&instances);

                        self.queue
                            .write_buffer(&self.instance_buffer, current_offset, data);

                        let byte_size = data.len() as u64;
                        calls.push((*mesh_id, instances.len(), current_offset, byte_size));
                        current_offset += byte_size;
                    }
                    calls
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
        self.profiler.record_duration("render/accumulate", accumulate_start.elapsed());
        let total_instances: usize = draw_calls.iter().map(|(_, count, ..)| count).sum();
        self.profiler.record_value("render/accumulate", "instances", total_instances as f64);

        let glyph_instances = if self.show_profiler_hud {
            self.build_hud_glyphs()
        } else {
            Vec::new()
        };
        if !glyph_instances.is_empty() {
            self.queue.write_buffer(
                &self.glyph_instance_buffer,
                0,
                bytemuck::cast_slice(&glyph_instances),
            );
        }

        self.upload_camera();

        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        let draw_start = std::time::Instant::now();

        renderpass.set_pipeline(&self.pipeline);
        renderpass.set_bind_group(0, &self.camera_bind_group, &[]);

        for (mesh_id, instance_count, offset, byte_size) in &draw_calls {
            let Some(gpu_mesh) = self.mesh_registry.get(mesh_id) else {
                continue;
            };

            renderpass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            renderpass.set_vertex_buffer(
                1,
                self.instance_buffer.slice(*offset..(*offset + *byte_size)),
            );
            renderpass.set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            renderpass.draw_indexed(0..gpu_mesh.index_count, 0, 0..*instance_count as u32);
        }

        if !glyph_instances.is_empty() {
            renderpass.set_pipeline(&self.text_pipeline);
            renderpass.set_bind_group(0, &self.text_screen_bind_group, &[]);
            renderpass.set_bind_group(1, &self.text_atlas_bind_group, &[]);
            renderpass.set_vertex_buffer(0, self.text_quad_vertex_buffer.slice(..));
            renderpass.set_vertex_buffer(1, self.glyph_instance_buffer.slice(..));
            renderpass.set_index_buffer(self.text_quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            renderpass.draw_indexed(0..6, 0, 0..glyph_instances.len() as u32);
        }

        drop(renderpass);

        self.profiler.record_duration("render/draw", draw_start.elapsed());
        self.profiler.record_value("render/draw", "draw_calls", draw_calls.len() as f64);

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();

        self.frame_count += 1;
        if self.last_time + 1000 < Utc::now().timestamp_millis() {
            println!("{} fps", self.frame_count);
            self.frame_count = 0;
            self.last_time = Utc::now().timestamp_millis();
        }
    }

    /// loads an obj file, uploads it to the gpu, and returns its MeshSourceId.
    /// calling with the same path twice returns the cached id without re-uploading.
    pub(crate) fn load_mesh_from_path(&mut self, path: &Path) -> anyhow::Result<MeshSourceId> {
        let owned = path.to_path_buf();

        if let Some(&id) = self.mesh_path_cache.get(&owned) {
            return Ok(id);
        }

        let (vertices, indices) = AssetReader::load_obj(path)?;

        self.mesh_id_counter += 1;
        let id = MeshSourceId(self.mesh_id_counter);

        self.upload_mesh(id, &vertices, &indices);
        self.mesh_path_cache.insert(owned, id);

        Ok(id)
    }

    /// creates a new scene. the first scene created becomes the active render target
    pub fn create_scene(&mut self, name: String) {
        self.scenes.insert(name.clone(), World::new());

        if self.current_scene.is_none() {
            self.current_scene = Some(name.clone());
        }

        println!("new scene of name {} created", name);
    }

    /// returns a shared reference to a scene by name
    pub fn get_scene(&self, name: String) -> Option<&World> {
        self.scenes.get(&name)
    }

    /// returns a mutable reference to a scene by name
    pub fn get_scene_mut(&mut self, name: String) -> Option<&mut World> {
        self.scenes.get_mut(&name)
    }
}
