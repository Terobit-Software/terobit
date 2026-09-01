use chrono::Utc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use glam::{Mat4, Vec3};
use glyphon::{
    Attrs, Buffer as TextBuffer, Cache as GlyphonCache, Color as TextColor, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{
    FrontFace, MultisampleState, PipelineCompilationOptions, PrimitiveState, PrimitiveTopology,
    Texture, TextureView, util::DeviceExt,
};
use winit::{event_loop::OwnedDisplayHandle, window::Window};

use crate::{
    assets::{AssetReader, MeshLoadOptions, mesh_simplify::simplify_mesh},
    consts::{
        MSAA_SAMPLE_COUNT, SHADOW_CASCADE_COUNT, SHADOW_MAP_SIZE, SHADOW_MAX_DISTANCE,
        SUN_DIRECTION_TO_LIGHT,
    },
    ecs::{Camera, PhysicsScene, Transform},
    profiling::Profiler,
    rendering::{
        CameraUniform, CullGlobals, CullingResources, DrawIndexedIndirectArgs, GpuMesh,
        InstanceRaw, LightUniform, MeshCullInfo, MeshSourceHandler, MeshSourceId, RenderCamera,
        ShadowCullSlot, Vertex, VertexRaw, VertexTrait, accumulate_meshes,
        compute_bounding_sphere, compute_cascades, extract_frustum_planes, taa_jitter,
    },
};
use bevy_ecs::prelude::*;
use bevy_ecs::world::World;

/// offscreen format the 3d scene renders into, before taa resolve. linear (not srgb)
/// so lighting math and the temporal blend both happen in linear space, matching the
/// history buffer's format
const SCENE_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// per-pixel screen-space motion, in uv units (current_uv - previous_uv)
const VELOCITY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
/// taa's ping-ponged history buffer, same format as the scene color it's resolved from
const HISTORY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// everything `init_text_rendering` builds, bundled up so it doesn't have to return
/// a big tuple
struct TextRenderResources {
    font_system: FontSystem,
    swash_cache: SwashCache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: TextRenderer,
    hud_buffer: TextBuffer,
    mesh_stats_buffer: TextBuffer,
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
    /// dis is how we render: 3d scene into scene_color + velocity, jittered for taa
    pipeline: wgpu::RenderPipeline,

    /// the camera buffer inside the camera_bind_group
    camera_buffer: wgpu::Buffer,
    /// the camera resource group for the gpu
    camera_bind_group: wgpu::BindGroup,

    /// the screen depth texture used for 3d rendering
    depth_texture: wgpu::Texture,
    /// the depth view, pointing to the depth_texture
    depth_view: wgpu::TextureView,

    /// offscreen target the 3d scene pass renders color into, before taa resolve
    scene_color_texture: wgpu::Texture,
    scene_color_view: wgpu::TextureView,
    /// per-pixel motion vectors written by the 3d scene pass, consumed by taa resolve
    velocity_texture: wgpu::Texture,
    velocity_view: wgpu::TextureView,
    /// taa's ping-ponged history buffer. `history_index` says which slot holds the
    /// most recently resolved frame (read from it, write the other one)
    history_textures: [wgpu::Texture; 2],
    history_views: [wgpu::TextureView; 2],
    history_index: usize,
    /// linear-filtered sampler used by the taa resolve pass for all its texture reads
    taa_sampler: wgpu::Sampler,
    /// fullscreen pass that reprojects history using the velocity buffer and neighborhood-
    /// clamps it against the current frame, writing the antialiased result to both the
    /// swapchain and next frame's history slot
    resolve_pipeline: wgpu::RenderPipeline,
    resolve_bind_group_layout: wgpu::BindGroupLayout,
    /// precomputed pair of resolve bind groups, indexed by `history_index` (the "read"
    /// slot for this frame) - rebuilt only on resize instead of every frame, since the
    /// views they reference only change then
    resolve_bind_groups: [wgpu::BindGroup; 2],
    /// unjittered view_proj from last frame, paired with this frame's in the camera
    /// uniform so the shader can reconstruct motion vectors
    prev_view_proj: Mat4,
    /// monotonic counter driving the halton jitter sequence, independent of `frame_count`
    /// (which resets every second for the fps counter)
    taa_frame_index: u32,
    /// each entity's model matrix as of last frame, used to compute per-instance motion
    /// vectors. persists frame to frame (see `accumulate_meshes`), so despawned
    /// entities leave a stale entry instead of being pruned automatically
    prev_transforms: HashMap<Entity, Mat4>,
    /// this frame's instances grouped by mesh id, reused frame to frame - `accumulate_meshes`
    /// clears each vec in place (keeping its allocation) instead of rebuilding the map
    mesh_instance_containers: HashMap<MeshSourceId, Vec<InstanceRaw>>,
    /// scratch buffer instance data is packed into (one mesh's instances contiguously
    /// after another) before a single `write_buffer` call uploads all of it at once,
    /// instead of one `write_buffer` call per mesh. reused frame to frame like the above
    instance_upload_scratch: Vec<InstanceRaw>,

    /// depth-only pipeline that renders one cascade's shadow map. no fragment stage
    shadow_pipeline: wgpu::RenderPipeline,
    /// one cascade per array layer, `Depth32Float`, sampled with a comparison sampler
    /// from the main pass
    shadow_map_texture: wgpu::Texture,
    /// single-layer views into `shadow_map_texture`, used as the shadow pass's depth
    /// attachment (one render pass per cascade)
    shadow_map_layer_views: [wgpu::TextureView; SHADOW_CASCADE_COUNT],
    shadow_sampler: wgpu::Sampler,
    /// one tiny buffer per cascade holding just that cascade's light view_proj, bound
    /// to the shadow pass. separate from `light_buffer` below (which holds all of them
    /// at once for the main pass) since the shadow pass only ever needs one at a time
    shadow_cascade_buffers: [wgpu::Buffer; SHADOW_CASCADE_COUNT],
    shadow_cascade_bind_groups: [wgpu::BindGroup; SHADOW_CASCADE_COUNT],
    /// every cascade's view_proj + split depth + light direction, for the main pass's
    /// shadow lookup (group 1)
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    /// shadow map texture array + comparison sampler, for the main pass (group 2)
    shadow_bind_group: wgpu::BindGroup,

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
    /// gpu-driven frustum + occlusion culling: compute pipelines, hi-z pyramid, and the
    /// buffers `cull.wgsl` reads/writes. main scene pass draws from
    /// `culling.visible_instance_buffer` instead of `instance_buffer` directly
    culling: CullingResources,
    /// one per shadow cascade: frustum-only gpu culling (no hi-z) against that
    /// cascade's light-space frustum instead of the main camera's, so the shadow pass
    /// can draw_indexed_indirect from a compacted buffer instead of looping the full
    /// unculled draw_calls list per cascade
    shadow_cull_slots: [ShadowCullSlot; SHADOW_CASCADE_COUNT],
    /// counter for assigning unique ids to custom meshes. starts at 1 (0 and 1 are
    /// reserved for the built-in cube and plane), so the first custom mesh gets id 2
    mesh_id_counter: usize,
    /// cache mapping file path to its registered MeshSourceId so we don't re-upload the same file
    // keyed on (path, load option bits) so the same file can live at several
    // detail levels at once
    mesh_path_cache: HashMap<(PathBuf, Option<(u32, u32)>), MeshSourceId>,
    /// cpu-side vertex positions per mesh, kept around so `Collider::inherit()` can
    /// build a physics shape from any mesh (built-in or asset-loaded)
    mesh_points: HashMap<MeshSourceId, Vec<Vec3>>,
    /// local-space bounding sphere per mesh, computed once at upload time from
    /// `mesh_points`, used by `cull.wgsl` to build each instance's world-space bound
    mesh_bounds: HashMap<MeshSourceId, (Vec3, f32)>,
    /// clock used to compute real per-frame dt, drives both `on_frame` and the physics step
    physics_clock: std::time::Instant,
    /// the physics simulation for the current scene
    physics: PhysicsScene,

    /// labeled timings/counters for engine internals, exposed to game code too via
    /// `RuntimeContext`. toggled on screen with f7+p
    profiler: Profiler,
    show_profiler_hud: bool,
    /// toggled on screen with f7+m
    show_mesh_stats_hud: bool,
    /// glyphon text stack, shared by both the profiler and mesh stats huds. viewport
    /// tracks screen size, atlas caches rasterized glyphs, renderer draws them into
    /// the shared render pass
    text_font_system: FontSystem,
    text_swash_cache: SwashCache,
    text_viewport: Viewport,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    /// re-shaped every frame the profiler hud is visible
    hud_text_buffer: TextBuffer,
    /// re-shaped every frame the mesh stats hud is visible
    mesh_stats_text_buffer: TextBuffer,
    /// (mesh_id, pre-cull instance count) snapshot taken the frame a gpu readback of
    /// post-cull counts was kicked off, in the same slot order as that readback. paired
    /// against the counts once `culling.take_readback()` resolves them a frame or two
    /// later, since resolution always lags the frame that started it
    pending_mesh_stats: Vec<(MeshSourceId, usize)>,
    /// post-cull instance count per mesh, from the most recently resolved gpu readback.
    /// mesh stats hud shows this instead of the current frame's count since that one
    /// isn't available on the cpu yet
    last_culled_counts: HashMap<MeshSourceId, u32>,

    /// gpu timestamp query pair (index 0 = begin, 1 = end) bracketing the culled scene
    /// pass, so the profiler hud can show actual gpu execution time for it instead of
    /// just the cpu time spent recording the draw calls (which barely varies with what's
    /// on screen - see `record_readback_copy`'s sibling below for why the culled count
    /// doesn't shrink the number of commands recorded either)
    scene_timestamp_query_set: wgpu::QuerySet,
    /// query_set's results, resolved into a plain (non-mappable) buffer first since
    /// resolve_query_set can't target a mappable one directly
    scene_timestamp_resolve_buffer: wgpu::Buffer,
    /// mappable copy of the resolve buffer, read back to the cpu
    scene_timestamp_readback_buffer: wgpu::Buffer,
    /// true while a previous frame's timestamp readback hasn't resolved yet
    scene_timestamp_pending: bool,
    /// filled in by the readback's map_async callback, [begin_ticks, end_ticks]
    scene_timestamp_result: Arc<Mutex<Option<[u64; 2]>>>,
    /// nanoseconds per gpu timestamp tick, queried once at startup, multiply a tick
    /// difference by this to get real elapsed time
    timestamp_period_ns: f32,
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

        // cull.wgsl's compacted draws use a nonzero first_instance (each mesh's
        // base_offset) in draw_indexed_indirect. without this feature wgpu's indirect
        // validation pass zeroes out any indirect draw whose first_instance isn't 0,
        // which silently skipped every mesh except whichever one happened to land at
        // offset 0 that frame, this is what was causing meshes to flicker in and out.
        // TIMESTAMP_QUERY lets the scene pass be bracketed with gpu timestamps for the
        // mesh stats hud's "real" render time, see scene_timestamp_query_set
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::INDIRECT_FIRST_INSTANCE
                    | wgpu::Features::TIMESTAMP_QUERY,
                ..Default::default()
            })
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

        // first frame has no history yet, so prev_view_proj starts equal to view_proj:
        // that gives every instance zero velocity instead of a spurious full-screen one
        let initial_view_proj = camera.view_proj_mat4(size.width, size.height);
        let initial_camera_uniform = CameraUniform {
            view_proj: initial_view_proj.to_cols_array_2d(),
            prev_view_proj: initial_view_proj.to_cols_array_2d(),
            view: camera.view_mat4().to_cols_array_2d(),
            jitter: [0.0, 0.0],
            _pad: [0.0, 0.0],
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::cast_slice(&[initial_camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // vertex reads view_proj/prev_view_proj/jitter; fragment also reads
                    // view (for view-space depth, used to pick a shadow cascade)
                    visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::FRAGMENT),
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

        // group 1: every cascade's light view_proj + split depths + light direction,
        // written once per frame in `render`
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("light buffer"),
            size: std::mem::size_of::<LightUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("light bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light bind group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        // group 2: the shadow map array + comparison sampler, sampled by the main pass
        let shadow_map_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow map texture"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: SHADOW_CASCADE_COUNT as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_map_array_view = shadow_map_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("shadow map array view"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let shadow_map_layer_views: [wgpu::TextureView; SHADOW_CASCADE_COUNT] =
            std::array::from_fn(|i| {
                shadow_map_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("shadow map layer view"),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i as u32,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            });

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow comparison sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            });

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow bind group"),
            layout: &shadow_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_map_array_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[
                Some(&camera_bind_group_layout),
                Some(&light_bind_group_layout),
                Some(&shadow_bind_group_layout),
            ],
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
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: SCENE_COLOR_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: VELOCITY_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            label: None,
            layout: Some(&pipeline_layout),
            primitive: PrimitiveState {
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                // every mesh winds its triangles ccw with outward-facing normals (see
                // the cube/plane builders in mesh_source.rs), so the rasterizer can
                // drop back faces for free instead of shading triangles that face away
                cull_mode: Some(wgpu::Face::Back),
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

        // depth-only pipeline that renders one cascade's shadow map. shares the same
        // vertex/instance buffers as the main pipeline (it only reads position + model
        // matrix out of them), just with its own much simpler shader and no fragment stage
        let shadow_shader_source = include_str!("shaders/shadow_depth.wgsl");
        let shadow_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow depth shader"),
            source: wgpu::ShaderSource::Wgsl(shadow_shader_source.into()),
        });

        let shadow_cascade_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow cascade bind group layout"),
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

        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[Some(&shadow_cascade_bind_group_layout)],
                ..Default::default()
            });

        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow depth pipeline"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader_module,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: PipelineCompilationOptions {
                    constants: &[],
                    zero_initialize_workgroup_memory: false,
                },
            },
            // depth-only: no color output, no fragment invocations at all
            fragment: None,
            primitive: PrimitiveState {
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                // render back faces instead of front faces for shadow casters: pushes
                // the occluding surface to the far side of each mesh, which avoids most
                // shadow acne without needing a depth bias tuned per-scene
                cull_mode: Some(wgpu::Face::Front),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            cache: None,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: 0xFFFFFFFF,
            },
            multiview_mask: None,
        });

        let shadow_cascade_buffers: [wgpu::Buffer; SHADOW_CASCADE_COUNT] =
            std::array::from_fn(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("shadow cascade matrix buffer"),
                    size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            });

        let shadow_cascade_bind_groups: [wgpu::BindGroup; SHADOW_CASCADE_COUNT] =
            std::array::from_fn(|i| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("shadow cascade bind group"),
                    layout: &shadow_cascade_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: shadow_cascade_buffers[i].as_entire_binding(),
                    }],
                })
            });

        // pre-allocate space for up to 500,001 instances. also readable as a storage
        // buffer since cull.wgsl reads it directly (compute doesn't go through the
        // vertex-buffer binding)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 500000) as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // gpu-culled counterpart: same capacity, written only by cull.wgsl, read as a
        // vertex buffer by the main scene pass
        let visible_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Visible Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 500000) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let culling = CullingResources::init(
            &device,
            size,
            &depth_view,
            &instance_buffer,
            visible_instance_buffer,
        );

        let shadow_cull_slots: [ShadowCullSlot; SHADOW_CASCADE_COUNT] =
            std::array::from_fn(|_| ShadowCullSlot::new(&device, &culling, &instance_buffer));

        // scene pass gpu timer (see scene_timestamp_query_set): 2 timestamps (begin, end),
        // each 8 bytes, resolved into a plain buffer then copied into a mappable one
        let scene_timestamp_query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("scene pass timestamp query set"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let scene_timestamp_resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene pass timestamp resolve buffer"),
            size: 2 * 8,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let scene_timestamp_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene pass timestamp readback buffer"),
            size: 2 * 8,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let timestamp_period_ns = queue.get_timestamp_period();

        let (scene_color_texture, scene_color_view) =
            Self::create_color_texture(&device, size, SCENE_COLOR_FORMAT, "scene color texture");
        let (velocity_texture, velocity_view) =
            Self::create_color_texture(&device, size, VELOCITY_FORMAT, "velocity texture");
        let (history_a_texture, history_a_view) =
            Self::create_color_texture(&device, size, HISTORY_FORMAT, "taa history texture a");
        let (history_b_texture, history_b_view) =
            Self::create_color_texture(&device, size, HISTORY_FORMAT, "taa history texture b");

        let taa_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("taa sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let resolve_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("taa resolve bind group layout"),
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
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let resolve_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[Some(&resolve_bind_group_layout)],
                ..Default::default()
            });

        let resolve_shader_source = include_str!("shaders/taa_resolve.wgsl");
        let resolve_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("taa resolve shader"),
            source: wgpu::ShaderSource::Wgsl(resolve_shader_source.into()),
        });

        let resolve_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("taa resolve pipeline"),
            layout: Some(&resolve_pipeline_layout),
            // fullscreen triangle generated from the vertex index, no vertex buffers
            vertex: wgpu::VertexState {
                module: &resolve_shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &resolve_shader_module,
                entry_point: Some("fs_main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: HISTORY_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
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
            cache: None,
            depth_stencil: None,
            multisample: MultisampleState {
                alpha_to_coverage_enabled: false,
                count: 1,
                mask: 0xFFFFFFFF,
            },
            multiview_mask: None,
        });

        let resolve_bind_groups = Self::build_resolve_bind_groups(
            &device,
            &resolve_bind_group_layout,
            &scene_color_view,
            &history_a_view,
            &history_b_view,
            &velocity_view,
            &taa_sampler,
        );

        let text = Self::init_text_rendering(&device, &queue, surface_format);

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
            scene_color_texture,
            scene_color_view,
            velocity_texture,
            velocity_view,
            history_textures: [history_a_texture, history_b_texture],
            history_views: [history_a_view, history_b_view],
            history_index: 0,
            taa_sampler,
            resolve_pipeline,
            resolve_bind_group_layout,
            resolve_bind_groups,
            prev_view_proj: initial_view_proj,
            taa_frame_index: 0,
            prev_transforms: HashMap::new(),
            mesh_instance_containers: HashMap::new(),
            instance_upload_scratch: Vec::new(),
            shadow_pipeline,
            shadow_map_texture,
            shadow_map_layer_views,
            shadow_sampler,
            shadow_cascade_buffers,
            shadow_cascade_bind_groups,
            light_buffer,
            light_bind_group,
            shadow_bind_group,
            camera,
            frame_count: 0,
            last_time: Utc::now().timestamp_millis(),
            scenes: HashMap::new(),
            current_scene: None,
            instance_buffer,
            culling,
            shadow_cull_slots,
            mesh_registry: HashMap::new(),
            mesh_id_counter: 1,
            mesh_path_cache: HashMap::new(),
            mesh_points: HashMap::new(),
            mesh_bounds: HashMap::new(),
            physics_clock: std::time::Instant::now(),
            physics: PhysicsScene::new(),
            profiler: Profiler::new(),
            show_profiler_hud: false,
            show_mesh_stats_hud: false,
            text_font_system: text.font_system,
            text_swash_cache: text.swash_cache,
            text_viewport: text.viewport,
            text_atlas: text.atlas,
            text_renderer: text.renderer,
            hud_text_buffer: text.hud_buffer,
            mesh_stats_text_buffer: text.mesh_stats_buffer,
            pending_mesh_stats: Vec::new(),
            last_culled_counts: HashMap::new(),
            scene_timestamp_query_set,
            scene_timestamp_resolve_buffer,
            scene_timestamp_readback_buffer,
            scene_timestamp_pending: false,
            scene_timestamp_result: Arc::new(Mutex::new(None)),
            timestamp_period_ns,
        };

        let res = MeshSourceHandler::init_default_meshes(&mut state);
        if res.is_err() {
            panic!("failed to upload default meshes");
        } else {
            println!("uploaded default meshes");
        }

        state.configure_surface();

        state
    }

    /// builds everything the profiler hud's text needs: glyphon's font system, glyph
    /// atlas, and renderer, plus the reusable text buffer the hud reshapes every frame
    fn init_text_rendering(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> TextRenderResources {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let cache = GlyphonCache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let renderer = TextRenderer::new(
            &mut atlas,
            device,
            MultisampleState {
                alpha_to_coverage_enabled: false,
                count: MSAA_SAMPLE_COUNT,
                mask: 0xFFFFFFFF,
            },
            // hud text gets its own render pass straight onto the swapchain (see
            // `render`), so unlike the old shared-pass setup it needs no depth attachment
            None,
        );

        let hud_buffer = TextBuffer::new(&mut font_system, Metrics::new(16.0, 18.0));
        let mesh_stats_buffer = TextBuffer::new(&mut font_system, Metrics::new(16.0, 18.0));

        TextRenderResources {
            font_system,
            swash_cache,
            viewport,
            atlas,
            mesh_stats_buffer,
            renderer,
            hud_buffer,
        }
    }

    /// pulls position and rotation off the first camera entity in the scene, if any
    fn update_camera_pose(&mut self) {
        let camera_data: Option<(Vec3, Vec3)> = if let Some(scene_name) = self.current_scene.clone()
        {
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
    }

    /// uploads vertex and index data for a mesh to the gpu and registers it by id.
    /// also keeps a cpu-side copy of the vertex positions so `Collider::inherit()`
    /// can build a physics shape from this mesh later
    pub fn upload_mesh(&mut self, id: MeshSourceId, vertices: &[VertexRaw], indices: &[u32]) {
        let points: Vec<Vec3> = vertices
            .iter()
            .map(|v| Vec3::new(v.position[0], v.position[1], v.position[2]))
            .collect();
        self.mesh_bounds
            .insert(id, compute_bounding_sphere(&points));
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
        let Some(scene_name) = self.current_scene.as_ref() else {
            return;
        };
        let Some(scene) = self.scenes.get_mut(scene_name) else {
            return;
        };
        self.physics
            .step(scene, dt, &self.mesh_points, &mut self.profiler);
    }

    /// engine-wide profiler, also reachable from game code through `RuntimeContext`
    pub(crate) fn profiler_mut(&mut self) -> &mut Profiler {
        &mut self.profiler
    }

    /// flips whether the f7+p profiler hud draws over the scene
    pub(crate) fn toggle_profiler_hud(&mut self) {
        self.show_profiler_hud = !self.show_profiler_hud;
    }

    /// flips whether the f7+m mesh stats hud draws over the scene
    pub(crate) fn toggle_mesh_stats_hud(&mut self) {
        self.show_mesh_stats_hud = !self.show_mesh_stats_hud;
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
            // TEXTURE_BINDING so the hi-z copy compute pass can textureLoad it
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        (depth_texture, depth_view)
    }

    /// creates a single-sample color texture usable both as a render target and a
    /// shader-readable texture. used for the scene color, velocity, and taa history buffers
    fn create_color_texture(
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> (Texture, TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        (texture, view)
    }

    /// builds both taa resolve bind groups up front - one per `history_index` parity -
    /// instead of recreating one every frame. only needs rebuilding when the views it
    /// references change, i.e. on resize; the ping-pong itself just picks one of the two
    #[allow(clippy::too_many_arguments)]
    fn build_resolve_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        scene_color_view: &TextureView,
        history_view_a: &TextureView,
        history_view_b: &TextureView,
        velocity_view: &TextureView,
        sampler: &wgpu::Sampler,
    ) -> [wgpu::BindGroup; 2] {
        let history_views = [history_view_a, history_view_b];
        std::array::from_fn(|read_history| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("taa resolve bind group"),
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(scene_color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(history_views[read_history]),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(velocity_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            })
        })
    }

    /// handles window resize: updates size, reconfigures surface, recreates every
    /// size-dependent texture (depth, scene color, velocity, taa history)
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();

        (self.depth_texture, self.depth_view) =
            Self::create_depth_texture(&self.device, new_size, MSAA_SAMPLE_COUNT);

        (self.scene_color_texture, self.scene_color_view) = Self::create_color_texture(
            &self.device,
            new_size,
            SCENE_COLOR_FORMAT,
            "scene color texture",
        );
        (self.velocity_texture, self.velocity_view) =
            Self::create_color_texture(&self.device, new_size, VELOCITY_FORMAT, "velocity texture");

        for i in 0..2 {
            (self.history_textures[i], self.history_views[i]) = Self::create_color_texture(
                &self.device,
                new_size,
                HISTORY_FORMAT,
                "taa history texture",
            );
        }

        // scene_color_view/velocity_view/history_views all just got recreated above -
        // rebuild the cached resolve bind groups that reference them
        self.resolve_bind_groups = Self::build_resolve_bind_groups(
            &self.device,
            &self.resolve_bind_group_layout,
            &self.scene_color_view,
            &self.history_views[0],
            &self.history_views[1],
            &self.velocity_view,
            &self.taa_sampler,
        );

        self.culling.resize(
            &self.device,
            new_size,
            &self.depth_view,
            &self.instance_buffer,
        );

        // culling's hi-z view changed above; each shadow cull slot's bind group
        // references it too (unused - occlusion is disabled - but still has to point at
        // a live view instead of leaking the old one)
        for slot in &mut self.shadow_cull_slots {
            slot.rebuild_bind_group(&self.device, &self.culling, &self.instance_buffer);
        }
    }

    /// builds the f7+p profiler hud text: one line per profiled label with its last/avg
    /// timing, plus any extra values attached to it (instance counts, draw calls, etc)
    fn build_hud_text(&self) -> String {
        let mut text = String::new();

        for (label, entry) in self.profiler.entries() {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&format!(
                "{}: {:.2}ms avg:{:.2}ms",
                label, entry.last_ms, entry.avg_ms
            ));

            let mut extra: Vec<_> = entry.values.iter().collect();
            extra.sort_by_key(|(key, _)| key.to_string());
            for (key, value) in extra {
                text.push_str(&format!(" {}:{:.0}", key, value));
            }
        }

        text
    }

    /// builds the f7+m mesh stats hud text: distinct mesh counts plus per-mesh
    /// pre-cull/post-cull instance counts. `draw_calls` is this frame's (mesh_id,
    /// instance_count, ..) list, used for the pre-cull side and the "drawn this frame"
    /// count; the post-cull side comes from `last_culled_counts`, which lags a frame or
    /// two behind since it's read back from the gpu (see `culling::CullingResources`)
    fn build_mesh_stats_text(&self, draw_calls: &[(MeshSourceId, usize, u64, u64)]) -> String {
        let pre_cull_total: usize = draw_calls.iter().map(|(_, count, ..)| count).sum();
        let post_cull_total: u32 = self.last_culled_counts.values().sum();

        let mut text = format!(
            "mesh types registered: {}\nmesh types drawn: {}\ninstances pre-cull: {}\ninstances post-cull: {} (lags ~1 frame)",
            self.mesh_registry.len(),
            draw_calls.len(),
            pre_cull_total,
            post_cull_total,
        );

        // draw_calls' order comes from a HashMap, not guaranteed stable frame to frame -
        // sort by id so the rows hold still
        let mut rows: Vec<&(MeshSourceId, usize, u64, u64)> = draw_calls.iter().collect();
        rows.sort_by_key(|(mesh_id, ..)| mesh_id.0);

        for (mesh_id, pre_count, ..) in rows {
            let post_count = self.last_culled_counts.get(mesh_id).copied().unwrap_or(0);
            text.push_str(&format!(
                "\n  mesh {}: {} -> {}",
                mesh_id.0, pre_count, post_count
            ));
        }

        text
    }

    /// renders the current frame: uploads instance data, submits draw calls, presents.
    ///
    /// one shadow depth pass per cascade, then the 3d scene (jittered, sampling those
    /// shadow maps) into an offscreen color + velocity buffer, then taa resolve
    /// reprojects last frame's history using those velocity vectors and neighborhood-
    /// clamps it against the current frame (writing the result to both the swapchain
    /// and next frame's history slot), then the hud text draws straight onto the
    /// swapchain, unjittered, so it stays crisp instead of getting blurred by taa
    pub fn render(&mut self) {
        // isolates surface acquisition (can stall on vsync/present queue depth) from
        // the rest of "render", so the profiler hud can tell actual cpu/gpu work apart
        // from time spent waiting on the display
        let acquire_start = std::time::Instant::now();
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
        self.profiler
            .record_duration("render/acquire", acquire_start.elapsed());

        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(self.surface_format.add_srgb_suffix()),
                ..Default::default()
            });

        let mut encoder = self.device.create_command_encoder(&Default::default());

        // updated early: both the shadow cascades below and the camera uniform later
        // in this function need the current frame's camera pose
        self.update_camera_pose();

        // computed once up front: reused for both this frame's cull frustum and,
        // further down, the camera uniform
        let current_view_proj = self
            .camera
            .view_proj_mat4(self.size.width, self.size.height);

        // draw calls and instance uploads happen before the renderpass starts. both
        // taken out of self for the duration of the scene borrow below (accumulate_meshes
        // updates them in place), and put back once that borrow ends
        let mut prev_transforms = std::mem::take(&mut self.prev_transforms);
        let mut mesh_instances = std::mem::take(&mut self.mesh_instance_containers);

        let accumulate_start = std::time::Instant::now();
        let draw_calls: Vec<(MeshSourceId, usize, u64, u64)> =
            if let Some(scene_name) = self.current_scene.clone() {
                if let Some(scene) = self.get_scene_mut(scene_name) {
                    accumulate_meshes(scene, &mut prev_transforms, &mut mesh_instances);

                    // pack every mesh's instances contiguously into one scratch buffer,
                    // then upload it in a single write_buffer call instead of one per
                    // mesh - each mesh's instances are already contiguous within their
                    // own Vec<InstanceRaw>, this just concatenates them in order
                    let mut upload_scratch = std::mem::take(&mut self.instance_upload_scratch);
                    upload_scratch.clear();

                    let mut calls = Vec::with_capacity(mesh_instances.len());
                    for (mesh_id, instances) in &mesh_instances {
                        if instances.is_empty() {
                            continue;
                        }

                        let offset = (upload_scratch.len() * std::mem::size_of::<InstanceRaw>())
                            as u64;
                        let byte_size =
                            (instances.len() * std::mem::size_of::<InstanceRaw>()) as u64;
                        upload_scratch.extend_from_slice(instances);
                        calls.push((*mesh_id, instances.len(), offset, byte_size));
                    }

                    if !upload_scratch.is_empty() {
                        self.queue.write_buffer(
                            &self.instance_buffer,
                            0,
                            bytemuck::cast_slice(&upload_scratch),
                        );
                    }

                    self.instance_upload_scratch = upload_scratch;
                    calls
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
        self.prev_transforms = prev_transforms;
        self.mesh_instance_containers = mesh_instances;
        self.profiler
            .record_duration("render/accumulate", accumulate_start.elapsed());
        let total_instances: usize = draw_calls.iter().map(|(_, count, ..)| count).sum();
        self.profiler
            .record_value("render/accumulate", "instances", total_instances as f64);

        // mesh stats hud (f7+m): drain whatever readback landed since last frame before
        // kicking off a new one below, pairing it with the mesh_id order captured back
        // when that readback started (not this frame's draw_calls, which may have since
        // reordered)
        let _ = self.device.poll(wgpu::PollType::Poll);
        if let Some(counts) = self.culling.take_readback() {
            self.last_culled_counts = self
                .pending_mesh_stats
                .iter()
                .zip(counts.iter())
                .map(|((mesh_id, _), &count)| (*mesh_id, count))
                .collect();
        }

        // scene pass gpu timer: same drain-before-kickoff deal as the readback above,
        // the poll() call already made just now covers both
        if let Some([begin, end]) = self.scene_timestamp_result.lock().unwrap().take() {
            self.scene_timestamp_pending = false;
            let ticks = end.saturating_sub(begin);
            let ms = ticks as f64 * self.timestamp_period_ns as f64 / 1_000_000.0;
            self.profiler
                .record_duration("render/scene_gpu", std::time::Duration::from_secs_f64(ms / 1000.0));
        }

        // shadow cascades: recomputed every frame from the current camera pose, since
        // both the camera's frustum slices and the stabilizing texel-snap depend on it.
        // computed here (rather than closer to where the cascade buffers get written,
        // further down) since the cull dispatches below need each cascade's frustum too
        let aspect = self.size.width as f32 / self.size.height as f32;
        let light_dir_to_light = Vec3::from(SUN_DIRECTION_TO_LIGHT).normalize();
        let cascades = compute_cascades(
            &self.camera,
            aspect,
            light_dir_to_light,
            SHADOW_MAX_DISTANCE,
            SHADOW_MAP_SIZE,
        );

        // gpu culling: one (mesh id, cull info, indirect args) entry per mesh drawn this
        // frame, in draw_calls order - that order becomes both buffers' dynamic/indirect
        // slot index, shared by write_frame_data/record_cull_dispatches below and the
        // main scene pass's indirect draw loop further down. the same mesh/instance
        // list (cull_info_and_args) is reused for each shadow cascade's own cull pass
        // below too - only the frustum being tested against differs per cascade
        let cull_start = std::time::Instant::now();
        let cull_entries: Vec<(MeshSourceId, MeshCullInfo, DrawIndexedIndirectArgs)> = draw_calls
            .iter()
            .filter_map(|(mesh_id, instance_count, offset, _byte_size)| {
                let gpu_mesh = self.mesh_registry.get(mesh_id)?;
                let (local_center, local_radius) = self
                    .mesh_bounds
                    .get(mesh_id)
                    .copied()
                    .unwrap_or((Vec3::ZERO, 0.0));
                let base_offset = (offset / std::mem::size_of::<InstanceRaw>() as u64) as u32;

                Some((
                    *mesh_id,
                    MeshCullInfo {
                        base_offset,
                        instance_count: *instance_count as u32,
                        local_radius,
                        _pad0: 0.0,
                        local_center: local_center.to_array(),
                        _pad1: 0.0,
                    },
                    DrawIndexedIndirectArgs {
                        index_count: gpu_mesh.index_count,
                        instance_count: 0,
                        first_index: 0,
                        base_vertex: 0,
                        first_instance: base_offset,
                    },
                ))
            })
            .collect();
        let cull_info_and_args: Vec<(MeshCullInfo, DrawIndexedIndirectArgs)> = cull_entries
            .iter()
            .map(|(_, info, args)| (*info, *args))
            .collect();

        let cull_globals = CullGlobals {
            frustum_planes: extract_frustum_planes(current_view_proj),
            prev_view_proj: self.prev_view_proj.to_cols_array_2d(),
            screen_size: [self.size.width as f32, self.size.height as f32],
            hiz_mip_count: self.culling.hiz_mip_count() as f32,
            near: self.camera.near,
            far: self.camera.far,
            _pad: [0.0, 0.0, 0.0],
        };
        self.culling
            .write_frame_data(&self.queue, &cull_info_and_args, &cull_globals);
        // hi-z build skipped: occlusion culling (the only consumer) is disabled, see
        // cull.wgsl. no point paying for the pyramid rebuild while nothing reads it

        // each cascade's frustum-cull output needs the same worst-case headroom as the
        // main pass's (see ShadowCullSlot) - grow now, before this frame's cull/draw
        // dispatches below reference these buffers
        for slot in &mut self.shadow_cull_slots {
            slot.ensure_capacity(
                &self.device,
                &self.culling,
                &self.instance_buffer,
                total_instances as u32,
            );
        }

        // per-cascade cull globals: same mesh/instance list as the main pass
        // (cull_info_and_args), just tested against that cascade's light-space frustum
        // instead of the camera's
        let shadow_cull_globals: [CullGlobals; SHADOW_CASCADE_COUNT] =
            std::array::from_fn(|i| CullGlobals {
                frustum_planes: extract_frustum_planes(cascades[i].view_proj),
                ..cull_globals
            });

        // cull dispatches get their own encoder + submit, separate from the shadow/scene
        // passes below. indirect_args_buffer/visible_instance_buffer are written here by
        // compute and read by draw_indexed_indirect later - that indirect-buffer read
        // isn't a regular bind-group usage, so it doesn't go through the same hazard
        // tracking as everything else. a submission boundary forces the (much more
        // heavily exercised) cross-submission ordering guarantee instead of relying on
        // same-encoder cross-pass tracking for this specific case. all 4 cascades'
        // cull dispatches ride along in this same submission boundary
        {
            let mut cull_encoder = self.device.create_command_encoder(&Default::default());
            self.culling
                .record_cull_dispatches(&mut cull_encoder, &cull_info_and_args);

            for (i, slot) in self.shadow_cull_slots.iter().enumerate() {
                self.culling.write_shadow_frame_data(
                    &self.queue,
                    &cull_info_and_args,
                    &shadow_cull_globals[i],
                    slot,
                );
                self.culling
                    .record_shadow_cull_dispatches(&mut cull_encoder, &cull_info_and_args, slot);
            }

            let readback_slot_count = cull_entries.len() as u32;
            self.culling
                .record_readback_copy(&mut cull_encoder, readback_slot_count);

            self.queue.submit([cull_encoder.finish()]);

            // mapping has to wait until after the copy above is actually submitted, see
            // `CullingResources::start_readback_map`
            if !self.culling.readback_pending() {
                self.culling.start_readback_map(readback_slot_count);
                self.pending_mesh_stats = cull_entries
                    .iter()
                    .map(|(mesh_id, info, _)| (*mesh_id, info.instance_count as usize))
                    .collect();
            }
        }
        self.profiler
            .record_duration("render/cull", cull_start.elapsed());

        self.text_viewport.update(
            &self.queue,
            Resolution {
                width: self.size.width,
                height: self.size.height,
            },
        );

        if self.show_profiler_hud || self.show_mesh_stats_hud {
            if self.show_profiler_hud {
                let hud_text = self.build_hud_text();
                self.hud_text_buffer.set_text(
                    &mut self.text_font_system,
                    &hud_text,
                    &Attrs::new(),
                    Shaping::Advanced,
                    None,
                );
            }
            if self.show_mesh_stats_hud {
                let mesh_stats_text = self.build_mesh_stats_text(&draw_calls);
                self.mesh_stats_text_buffer.set_text(
                    &mut self.text_font_system,
                    &mesh_stats_text,
                    &Attrs::new(),
                    Shaping::Advanced,
                    None,
                );
            }

            let mut text_areas = Vec::with_capacity(2);
            if self.show_profiler_hud {
                text_areas.push(TextArea {
                    buffer: &self.hud_text_buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds::default(),
                    default_color: TextColor::rgb(51, 255, 102),
                    custom_glyphs: &[],
                });
            }
            // to the right of the profiler hud so both can be up at once without overlap
            if self.show_mesh_stats_hud {
                text_areas.push(TextArea {
                    buffer: &self.mesh_stats_text_buffer,
                    left: 420.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds::default(),
                    default_color: TextColor::rgb(102, 204, 255),
                    custom_glyphs: &[],
                });
            }

            self.text_renderer
                .prepare(
                    &self.device,
                    &self.queue,
                    &mut self.text_font_system,
                    &mut self.text_atlas,
                    &self.text_viewport,
                    text_areas,
                    &mut self.text_swash_cache,
                )
                .unwrap();
        }

        // current frame's (unjittered) view_proj, paired with last frame's for the
        // shader's motion-vector math, plus this frame's sub-pixel jitter offset
        let jitter = taa_jitter(self.taa_frame_index, self.size.width, self.size.height);
        let camera_uniform = CameraUniform {
            view_proj: current_view_proj.to_cols_array_2d(),
            prev_view_proj: self.prev_view_proj.to_cols_array_2d(),
            view: self.camera.view_mat4().to_cols_array_2d(),
            jitter,
            _pad: [0.0, 0.0],
        };
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
        self.prev_view_proj = current_view_proj;
        self.taa_frame_index = self.taa_frame_index.wrapping_add(1);

        for (i, cascade) in cascades.iter().enumerate() {
            self.queue.write_buffer(
                &self.shadow_cascade_buffers[i],
                0,
                bytemuck::cast_slice(&[cascade.view_proj.to_cols_array_2d()]),
            );
        }

        let light_uniform = LightUniform {
            cascade_view_proj: cascades.map(|c| c.view_proj.to_cols_array_2d()),
            cascade_splits: cascades.map(|c| c.split_depth),
            light_dir: light_dir_to_light.to_array(),
            _pad: 0.0,
        };
        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[light_uniform]),
        );

        let draw_start = std::time::Instant::now();

        // shadow passes: one depth-only render per cascade, gpu-culled against that
        // cascade's own light-space frustum (see shadow_cull_slots above) instead of
        // looping the full unculled draw_calls list - same draw_indexed_indirect
        // pattern as the main scene pass below, just against that cascade's compacted
        // buffers instead of the main camera's
        for (i, layer_view) in self.shadow_map_layer_views.iter().enumerate() {
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow cascade pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: layer_view,
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

            shadow_pass.set_pipeline(&self.shadow_pipeline);
            shadow_pass.set_bind_group(0, &self.shadow_cascade_bind_groups[i], &[]);

            let cull_slot = &self.shadow_cull_slots[i];
            for (draw_slot, (mesh_id, _, _)) in cull_entries.iter().enumerate() {
                let Some(gpu_mesh) = self.mesh_registry.get(mesh_id) else {
                    continue;
                };

                shadow_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                shadow_pass.set_vertex_buffer(1, cull_slot.visible_instance_buffer.slice(..));
                shadow_pass
                    .set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                let indirect_offset =
                    draw_slot as u64 * self.culling.indirect_args_stride() as u64;
                shadow_pass
                    .draw_indexed_indirect(&cull_slot.indirect_args_buffer, indirect_offset);
            }
        }

        // pass a: 3d scene, jittered, rendered into offscreen color + velocity targets
        {
            let mut scene_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.scene_color_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.velocity_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                // only meaningful while a previous readback isn't still in flight -
                // writing fresh timestamps into a query set whose last resolve hasn't
                // been copied out yet would just get overwritten before it's read
                timestamp_writes: (!self.scene_timestamp_pending).then_some(
                    wgpu::RenderPassTimestampWrites {
                        query_set: &self.scene_timestamp_query_set,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    },
                ),
                occlusion_query_set: None,
                multiview_mask: None,
            });

            scene_pass.set_pipeline(&self.pipeline);
            scene_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            scene_pass.set_bind_group(1, &self.light_bind_group, &[]);
            scene_pass.set_bind_group(2, &self.shadow_bind_group, &[]);

            // gpu-culled: instance counts live in indirect_args_buffer (written by
            // cull.wgsl), not read back to the cpu. visible_instance_buffer is bound
            // whole every time since each mesh's surviving instances are already
            // compacted into its own segment (first_instance carries the offset)
            for (slot, (mesh_id, _, _)) in cull_entries.iter().enumerate() {
                let Some(gpu_mesh) = self.mesh_registry.get(mesh_id) else {
                    continue;
                };

                scene_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                scene_pass.set_vertex_buffer(1, self.culling.visible_instance_buffer.slice(..));
                scene_pass
                    .set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                let indirect_offset = slot as u64 * self.culling.indirect_args_stride() as u64;
                scene_pass
                    .draw_indexed_indirect(self.culling.indirect_args_buffer(), indirect_offset);
            }
        }

        // scene pass gpu timer: resolve this frame's timestamps (if any were written
        // above) into the mappable readback buffer. the async map itself has to wait
        // until after `encoder` is submitted at the end of this function - mapping a
        // buffer a not-yet-submitted encoder still writes to is a validation error, see
        // culling::CullingResources::start_readback_map for the same issue
        let scene_timestamps_recorded = !self.scene_timestamp_pending;
        if scene_timestamps_recorded {
            encoder.resolve_query_set(
                &self.scene_timestamp_query_set,
                0..2,
                &self.scene_timestamp_resolve_buffer,
                0,
            );
            encoder.copy_buffer_to_buffer(
                &self.scene_timestamp_resolve_buffer,
                0,
                &self.scene_timestamp_readback_buffer,
                0,
                2 * 8,
            );
        }

        // pass b: taa resolve. reads this frame's scene_color/velocity plus last
        // frame's resolved history, writes the antialiased result to both the
        // swapchain and the other history slot (ping-ponged for next frame)
        let read_history = self.history_index;
        let write_history = 1 - self.history_index;

        // precomputed at init/resize (see build_resolve_bind_groups) instead of
        // recreated here every frame - only the views it references change, and those
        // only change on resize
        let resolve_bind_group = &self.resolve_bind_groups[read_history];

        {
            let mut resolve_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("taa resolve pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.history_views[write_history],
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            resolve_pass.set_pipeline(&self.resolve_pipeline);
            resolve_pass.set_bind_group(0, resolve_bind_group, &[]);
            resolve_pass.draw(0..3, 0..1);
        }

        self.history_index = write_history;

        // pass c: hud text, straight onto the swapchain and unjittered, so taa never
        // blurs it
        if self.show_profiler_hud || self.show_mesh_stats_hud {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud text pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.text_renderer
                .render(&self.text_atlas, &self.text_viewport, &mut text_pass)
                .unwrap();
        }

        self.profiler
            .record_duration("render/draw", draw_start.elapsed());
        self.profiler
            .record_value("render/draw", "draw_calls", draw_calls.len() as f64);

        self.queue.submit([encoder.finish()]);

        // now that the resolve+copy above is actually submitted, it's safe to map the
        // readback buffer - see the comment where scene_timestamps_recorded is set
        if scene_timestamps_recorded {
            self.scene_timestamp_pending = true;
            let buffer = self.scene_timestamp_readback_buffer.clone();
            let result = self.scene_timestamp_result.clone();
            self.scene_timestamp_readback_buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |map_result| {
                    if map_result.is_err() {
                        return;
                    }
                    let data = buffer.slice(..).get_mapped_range();
                    let begin = bytemuck::pod_read_unaligned::<u64>(&data[0..8]);
                    let end = bytemuck::pod_read_unaligned::<u64>(&data[8..16]);
                    drop(data);
                    buffer.unmap();
                    *result.lock().unwrap() = Some([begin, end]);
                });
        }

        let present_start = std::time::Instant::now();
        self.window.pre_present_notify();
        surface_texture.present();
        self.profiler
            .record_duration("render/present", present_start.elapsed());

        self.frame_count += 1;
        if self.last_time + 1000 < Utc::now().timestamp_millis() {
            println!("{} fps", self.frame_count);
            self.frame_count = 0;
            self.last_time = Utc::now().timestamp_millis();
        }
    }

    /// loads an obj file, uploads it to the gpu, and returns its MeshSourceId.
    /// calling with the same path and options twice returns the cached id without
    /// re-uploading. simplification (if requested in `options`) happens here on the
    /// parsed vertex data, before upload, so the full-detail mesh never hits the gpu
    pub(crate) fn load_mesh_from_path(
        &mut self,
        path: &Path,
        options: MeshLoadOptions,
    ) -> anyhow::Result<MeshSourceId> {
        let key = (path.to_path_buf(), options.cache_key_bits());

        if let Some(&id) = self.mesh_path_cache.get(&key) {
            return Ok(id);
        }

        let (mut vertices, mut indices) = AssetReader::load_obj(path)?;

        if let Some(ratio) = options.simplify_ratio {
            let before = indices.len() / 3;
            (vertices, indices) =
                simplify_mesh(&vertices, &indices, ratio, options.simplify_target_error)?;
            println!(
                "simplified {:?}: {} -> {} tris",
                path,
                before,
                indices.len() / 3
            );
        }

        self.mesh_id_counter += 1;
        let id = MeshSourceId(self.mesh_id_counter);

        self.upload_mesh(id, &vertices, &indices);
        self.mesh_path_cache.insert(key, id);

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
