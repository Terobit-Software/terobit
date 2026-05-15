use chrono::Utc;
use std::{collections::HashMap, sync::Arc};

use glam::{Mat4, Vec2, Vec3};
use wgpu::{
    BufferSlice, Device, FrontFace, MultisampleState, PipelineCompilationOptions, PrimitiveState,
    PrimitiveTopology, Texture, TextureView, util::DeviceExt, wgc::device,
};
use winit::{event_loop::OwnedDisplayHandle, window::Window};

use crate::{
    consts::{DEFAULT_CAMERA_Z_CLOSE, DEFAULT_CAMERA_Z_FAR, DEFAULT_FOV, MSAA_SAMPLE_COUNT}, ecs::{Camera, Transform}, rendering::{
        GpuMesh, InstanceRaw, MeshSourceHandler, MeshSourceId, RenderCamera, Vertex, VertexRaw,
        VertexTrait, accumulate_meshes,
    }
};
use bevy_ecs::world::World;
use bevy_ecs::{prelude::*, world::WorldId};

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
}

impl AppState {
    /// create a new app state
    pub async fn new(display: OwnedDisplayHandle, window: Arc<Window>) -> Self {
        // create a new wgpu instance, using the display handle from winit
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::debugging(), // add this
            ..wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display))
        });

        // get the adapter, which represents the gpu
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        // get an actual connection to the gpu
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        // get the size of the window
        let size = window.inner_size();

        // create a surface to render to for the window
        let surface = instance.create_surface(window.clone()).unwrap();
        // represents how we can interact with the gpu, formats mostly
        let cap = surface.get_capabilities(&adapter);
        // gets the formats
        let surface_format = cap.formats[0];

        let shader_source = include_str!("shaders/root.wgsl");
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("root shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Model: identity (cube is already in world space)
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

        // pre-allocate space for up to 1024 instances
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (std::mem::size_of::<InstanceRaw>() * 1024) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // creates the actual app state
        let mut state = Self {
            instance,       // wgpu itself
            window,         // the window
            device,         // the gpu
            queue,          // to send work to the gpu
            size,           // the size of the surface
            surface,        // the surface itself
            surface_format, // the format of the surface (?)
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
        };

        let res = MeshSourceHandler::init_default_meshes(&mut state);
        if res.is_err() {
            panic!("failed to upload default meshes");
        } else {
            println!("uploaded default meshes");
        }

        state.upload_camera();

        // configures the surface of the state
        state.configure_surface();

        state
    }

    fn upload_camera(&mut self) {
        let camera_position: Option<Vec3> = if let Some(scene_name) = self.current_scene.clone() {
            if let Some(scene) = self.get_scene_mut(scene_name) {
                scene.query::<(&Transform, &Camera)>()
                    .iter(scene)
                    .next() // just the first camera
                    .map(|(transform, _)| transform.position)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(pos) = camera_position {
            self.camera.position = pos;
        }
        
        let mvp = self
            .camera
            .build_view_proj(self.size.width, self.size.height);
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[mvp]));
    }

    pub fn upload_mesh(&mut self, id: MeshSourceId, vertices: &[VertexRaw], indices: &[u32]) {
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

    pub fn configure_surface(&self) {
        // creates a surface config
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

    pub fn get_window(&self) -> &Window {
        &self.window
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

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
        (self.depth_texture, self.depth_view) =
            Self::create_depth_texture(&self.device, new_size, MSAA_SAMPLE_COUNT);

        self.upload_camera();
    }
    
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

    
        // collect draw calls and write instance data before renderpass
        let draw_calls: Vec<(MeshSourceId, usize, u64, u64)> =
            if let Some(scene_name) = self.current_scene.clone() {
                if let Some(scene) = self.get_scene_mut(scene_name) {
                    let containers = accumulate_meshes(scene);
                    let mut current_offset: u64 = 0;
                    let mut calls = vec![];
    
                    for (mesh_id, container) in &containers {
                        let instances: Vec<InstanceRaw> = container.transforms
                            .iter()
                            .map(|t| InstanceRaw::new(*t))
                            .collect();
                        let data: &[u8] = bytemuck::cast_slice(&instances);
    
                        self.queue.write_buffer(&self.instance_buffer, current_offset, data);
    
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

        self.upload_camera();
    
        let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
    
        renderpass.set_pipeline(&self.pipeline);
        renderpass.set_bind_group(0, &self.camera_bind_group, &[]);
    
        for (mesh_id, instance_count, offset, byte_size) in &draw_calls {
            let Some(gpu_mesh) = self.mesh_registry.get(mesh_id) else {
                continue;
            };
    
            renderpass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            renderpass.set_vertex_buffer(1, self.instance_buffer.slice(*offset..(*offset + *byte_size)));
            renderpass.set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            renderpass.draw_indexed(0..gpu_mesh.index_count, 0, 0..*instance_count as u32);
        }
    
        drop(renderpass);
    
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
    pub fn create_scene(&mut self, name: String) {
        self.scenes.insert(name.clone(), World::new());

        if self.current_scene.is_none() {
            self.current_scene = Some(name.clone());
        }

        println!("new scene of name {} created", name);
    }

    pub fn get_scene(&self, name: String) -> Option<&World> {
        self.scenes.get(&name)
    }

    pub fn get_scene_mut(&mut self, name: String) -> Option<&mut World> {
        self.scenes.get_mut(&name)
    }
}
