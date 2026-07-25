use std::sync::{Arc, Mutex};

use glam::Vec3;
use winit::dpi::PhysicalSize;

use crate::consts::MAX_CULL_MESH_SLOTS;

/// bounding sphere (local space) that conservatively wraps every point in a mesh.
/// center comes from the AABB midpoint, radius is the farthest point from that center,
/// so it's a bit looser than a minimal enclosing sphere but cheap to compute and good
/// enough for culling margins
pub(crate) fn compute_bounding_sphere(points: &[Vec3]) -> (Vec3, f32) {
    if points.is_empty() {
        return (Vec3::ZERO, 0.0);
    }

    let mut min = points[0];
    let mut max = points[0];
    for &p in points.iter().skip(1) {
        min = min.min(p);
        max = max.max(p);
    }

    let center = (min + max) * 0.5;
    let radius = points
        .iter()
        .map(|&p| (p - center).length())
        .fold(0.0_f32, f32::max);

    (center, radius)
}

/// per-mesh culling input, written fresh every frame into a dynamic-offset uniform
/// buffer (one slot per mesh drawn this frame). `base_offset` is an element index
/// (not bytes) into both `instance_buffer` and `visible_instance_buffer`
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MeshCullInfo {
    pub base_offset: u32,
    pub instance_count: u32,
    pub local_radius: f32,
    pub _pad0: f32,
    pub local_center: [f32; 3],
    pub _pad1: f32,
}

/// frustum planes + last frame's view_proj, shared by every mesh's cull dispatch this
/// frame. plane format is (nx, ny, nz, d) with the convention `dot(n, p) + d >= -radius`
/// meaning "inside or intersecting"
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct CullGlobals {
    pub frustum_planes: [[f32; 4]; 6],
    pub prev_view_proj: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub hiz_mip_count: f32,
    /// camera near/far, needed to linearize hi-z depth for the occlusion bias (see
    /// cull.wgsl) - comparing raw ndc z would make the bias margin meaningless near the
    /// far plane, where depth precision is compressed to almost nothing
    pub near: f32,
    pub far: f32,
    pub _pad: [f32; 3],
}

/// extracts the 6 view-frustum planes from a combined view_proj matrix (Gribb/Hartmann
/// method: each plane is a row combination of the matrix). normals point inward
pub(crate) fn extract_frustum_planes(view_proj: glam::Mat4) -> [[f32; 4]; 6] {
    let m = view_proj.to_cols_array_2d(); // m[col][row]
    let row = |r: usize| Vec3::new(m[0][r], m[1][r], m[2][r]).extend(m[3][r]);

    let r0 = row(0);
    let r1 = row(1);
    let r2 = row(2);
    let r3 = row(3);

    let planes = [
        r3 + r0, // left
        r3 - r0, // right
        r3 + r1, // bottom
        r3 - r1, // top
        r2,      // near (wgpu clip space z in [0, 1])
        r3 - r2, // far
    ];

    planes.map(|p| {
        let n = Vec3::new(p.x, p.y, p.z);
        let len = n.length().max(1e-8);
        (p / len).to_array()
    })
}

/// mirrors wgpu's expected layout for `draw_indexed_indirect`'s argument buffer exactly:
/// index_count, instance_count, first_index, base_vertex, first_instance
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct DrawIndexedIndirectArgs {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

/// rounds `value` up to the next multiple of `align` (`align` must be a power of two,
/// which every gpu buffer offset alignment wgpu reports is)
fn align_up(value: u32, align: u32) -> u32 {
    (value + align - 1) & !(align - 1)
}

/// mip levels needed so the smallest level is 1x1
fn mip_count_for(size: PhysicalSize<u32>) -> u32 {
    let max_dim = size.width.max(size.height).max(1);
    32 - max_dim.leading_zeros()
}

fn create_hiz_texture(
    device: &wgpu::Device,
    size: PhysicalSize<u32>,
) -> (wgpu::Texture, Vec<wgpu::TextureView>, wgpu::TextureView) {
    let mip_count = mip_count_for(size);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("hi-z pyramid"),
        size: wgpu::Extent3d {
            width: size.width.max(1),
            height: size.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
        view_formats: &[],
    });

    let mip_views: Vec<wgpu::TextureView> = (0..mip_count)
        .map(|i| {
            texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("hi-z mip view"),
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_mip_level: i,
                mip_level_count: Some(1),
                ..Default::default()
            })
        })
        .collect();

    let full_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("hi-z full chain view"),
        dimension: Some(wgpu::TextureViewDimension::D2),
        ..Default::default()
    });

    (texture, mip_views, full_view)
}

/// gpu-driven frustum + occlusion culling: the hi-z pyramid built from last frame's
/// depth buffer, the compute pipelines that build and consume it, and the buffers
/// `cull.wgsl` reads instances from / writes survivors into
pub(crate) struct CullingResources {
    pub visible_instance_buffer: wgpu::Buffer,
    indirect_args_buffer: wgpu::Buffer,
    indirect_args_stride: u32,
    mesh_cull_info_buffer: wgpu::Buffer,
    mesh_cull_info_stride: u32,
    cull_globals_buffer: wgpu::Buffer,

    hiz_mip_views: Vec<wgpu::TextureView>,
    hiz_sampler: wgpu::Sampler,

    hiz_copy_pipeline: wgpu::ComputePipeline,
    hiz_copy_bind_group_layout: wgpu::BindGroupLayout,
    hiz_copy_bind_group: wgpu::BindGroup,

    hiz_downsample_pipeline: wgpu::ComputePipeline,
    hiz_downsample_bind_group_layout: wgpu::BindGroupLayout,
    hiz_downsample_bind_groups: Vec<wgpu::BindGroup>,

    cull_pipeline: wgpu::ComputePipeline,
    cull_bind_group_layout: wgpu::BindGroupLayout,
    cull_bind_group: wgpu::BindGroup,

    /// mesh stats hud (f7+m): mappable copy of `indirect_args_buffer`, read back to the
    /// cpu each frame to show post-cull instance counts. separate from the gpu-only
    /// buffer above since that one isn't map-readable
    readback_buffer: wgpu::Buffer,
    /// filled in by `record_readback`'s map_async callback once the copy lands. shared
    /// with the callback closure, which may run on a different thread depending on backend
    readback_result: Arc<Mutex<Option<Vec<u32>>>>,
    /// true while a previously kicked-off readback hasn't resolved yet. mapping a
    /// buffer that's already mapped panics, so a new readback has to wait for it
    readback_pending: bool,
}

impl CullingResources {
    pub(crate) fn init(
        device: &wgpu::Device,
        size: PhysicalSize<u32>,
        depth_view: &wgpu::TextureView,
        instance_buffer: &wgpu::Buffer,
        visible_instance_buffer: wgpu::Buffer,
    ) -> Self {
        let limits = device.limits();
        let indirect_args_stride = align_up(
            std::mem::size_of::<DrawIndexedIndirectArgs>() as u32,
            limits.min_storage_buffer_offset_alignment,
        );
        let mesh_cull_info_stride = align_up(
            std::mem::size_of::<MeshCullInfo>() as u32,
            limits.min_uniform_buffer_offset_alignment,
        );

        let indirect_args_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("indirect draw args"),
            size: indirect_args_stride as u64 * MAX_CULL_MESH_SLOTS as u64,
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                // read back by the mesh stats hud (f7+m), see record_readback
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cull readback buffer"),
            size: indirect_args_stride as u64 * MAX_CULL_MESH_SLOTS as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mesh_cull_info_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh cull info"),
            size: mesh_cull_info_stride as u64 * MAX_CULL_MESH_SLOTS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cull_globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cull globals"),
            size: std::mem::size_of::<CullGlobals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let (_hiz_texture, hiz_mip_views, hiz_full_view) = create_hiz_texture(device, size);

        let hiz_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hi-z sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let hiz_copy_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hi-z copy shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../app/shaders/hiz_copy.wgsl").into()),
        });
        let hiz_copy_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hi-z copy bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let hiz_copy_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("hi-z copy pipeline layout"),
                bind_group_layouts: &[Some(&hiz_copy_bind_group_layout)],
                ..Default::default()
            });
        let hiz_copy_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("hi-z copy pipeline"),
            layout: Some(&hiz_copy_pipeline_layout),
            module: &hiz_copy_shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let hiz_copy_bind_group = Self::build_hiz_copy_bind_group(
            device,
            &hiz_copy_bind_group_layout,
            depth_view,
            &hiz_mip_views,
        );

        let hiz_downsample_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hi-z downsample shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../app/shaders/hiz_downsample.wgsl").into(),
            ),
        });
        let hiz_downsample_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("hi-z downsample bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::R32Float,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });
        let hiz_downsample_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("hi-z downsample pipeline layout"),
                bind_group_layouts: &[Some(&hiz_downsample_bind_group_layout)],
                ..Default::default()
            });
        let hiz_downsample_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("hi-z downsample pipeline"),
                layout: Some(&hiz_downsample_pipeline_layout),
                module: &hiz_downsample_shader,
                entry_point: Some("cs_main"),
                compilation_options: Default::default(),
                cache: None,
            });
        let hiz_downsample_bind_groups = Self::build_hiz_downsample_bind_groups(
            device,
            &hiz_downsample_bind_group_layout,
            &hiz_mip_views,
        );

        let cull_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("cull shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../app/shaders/cull.wgsl").into()),
        });
        let cull_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("cull bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: true,
                            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                                DrawIndexedIndirectArgs,
                            >(
                            )
                                as u64),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                                MeshCullInfo,
                            >(
                            )
                                as u64),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let cull_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cull pipeline layout"),
            bind_group_layouts: &[Some(&cull_bind_group_layout)],
            ..Default::default()
        });
        let cull_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("cull pipeline"),
            layout: Some(&cull_pipeline_layout),
            module: &cull_shader,
            entry_point: Some("cs_main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let cull_bind_group = Self::build_cull_bind_group(
            device,
            &cull_bind_group_layout,
            instance_buffer,
            &visible_instance_buffer,
            &indirect_args_buffer,
            &mesh_cull_info_buffer,
            &cull_globals_buffer,
            &hiz_full_view,
            &hiz_sampler,
        );

        Self {
            visible_instance_buffer,
            indirect_args_buffer,
            indirect_args_stride,
            mesh_cull_info_buffer,
            mesh_cull_info_stride,
            cull_globals_buffer,
            hiz_mip_views,
            hiz_sampler,
            hiz_copy_pipeline,
            hiz_copy_bind_group_layout,
            hiz_copy_bind_group,
            hiz_downsample_pipeline,
            hiz_downsample_bind_group_layout,
            hiz_downsample_bind_groups,
            cull_pipeline,
            cull_bind_group_layout,
            cull_bind_group,
            readback_buffer,
            readback_result: Arc::new(Mutex::new(None)),
            readback_pending: false,
        }
    }

    fn build_hiz_copy_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        depth_view: &wgpu::TextureView,
        hiz_mip_views: &[wgpu::TextureView],
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hi-z copy bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&hiz_mip_views[0]),
                },
            ],
        })
    }

    fn build_hiz_downsample_bind_groups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        hiz_mip_views: &[wgpu::TextureView],
    ) -> Vec<wgpu::BindGroup> {
        (1..hiz_mip_views.len())
            .map(|i| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("hi-z downsample bind group"),
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&hiz_mip_views[i - 1]),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&hiz_mip_views[i]),
                        },
                    ],
                })
            })
            .collect()
    }

    #[allow(clippy::too_many_arguments)]
    fn build_cull_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        instance_buffer: &wgpu::Buffer,
        visible_instance_buffer: &wgpu::Buffer,
        indirect_args_buffer: &wgpu::Buffer,
        mesh_cull_info_buffer: &wgpu::Buffer,
        cull_globals_buffer: &wgpu::Buffer,
        hiz_full_view: &wgpu::TextureView,
        hiz_sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cull bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: visible_instance_buffer.as_entire_binding(),
                },
                // dynamic-offset bindings: the bound size here is the size of one
                // mesh's slot, not the whole buffer, or offsetting into it would
                // overrun (offset + size must stay within the buffer)
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: indirect_args_buffer,
                        offset: 0,
                        size: std::num::NonZeroU64::new(
                            std::mem::size_of::<DrawIndexedIndirectArgs>() as u64,
                        ),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: mesh_cull_info_buffer,
                        offset: 0,
                        size: std::num::NonZeroU64::new(std::mem::size_of::<MeshCullInfo>() as u64),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: cull_globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(hiz_full_view),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(hiz_sampler),
                },
            ],
        })
    }

    /// recreates the hi-z pyramid at the new size and every bind group that references
    /// it (or the depth view, which also gets recreated on resize)
    pub(crate) fn resize(
        &mut self,
        device: &wgpu::Device,
        size: PhysicalSize<u32>,
        depth_view: &wgpu::TextureView,
        instance_buffer: &wgpu::Buffer,
    ) {
        let (_hiz_texture, hiz_mip_views, hiz_full_view) = create_hiz_texture(device, size);

        self.hiz_copy_bind_group = Self::build_hiz_copy_bind_group(
            device,
            &self.hiz_copy_bind_group_layout,
            depth_view,
            &hiz_mip_views,
        );
        self.hiz_downsample_bind_groups = Self::build_hiz_downsample_bind_groups(
            device,
            &self.hiz_downsample_bind_group_layout,
            &hiz_mip_views,
        );
        self.cull_bind_group = Self::build_cull_bind_group(
            device,
            &self.cull_bind_group_layout,
            instance_buffer,
            &self.visible_instance_buffer,
            &self.indirect_args_buffer,
            &self.mesh_cull_info_buffer,
            &self.cull_globals_buffer,
            &hiz_full_view,
            &self.hiz_sampler,
        );

        self.hiz_mip_views = hiz_mip_views;
    }

    /// rewrites this frame's per-mesh cull info + resets each mesh's indirect draw
    /// args, plus the shared frustum/hi-z globals. `meshes[slot]` must be in the same
    /// order as the draw calls that follow, since slot index doubles as the dynamic
    /// buffer offset used by `record_cull_dispatches`
    pub(crate) fn write_frame_data(
        &self,
        queue: &wgpu::Queue,
        meshes: &[(MeshCullInfo, DrawIndexedIndirectArgs)],
        globals: &CullGlobals,
    ) {
        if !meshes.is_empty() {
            let mut indirect_bytes = vec![0u8; self.indirect_args_stride as usize * meshes.len()];
            let mut mesh_info_bytes = vec![0u8; self.mesh_cull_info_stride as usize * meshes.len()];

            for (slot, (info, args)) in meshes.iter().enumerate() {
                let indirect_offset = slot * self.indirect_args_stride as usize;
                let indirect_size = std::mem::size_of::<DrawIndexedIndirectArgs>();
                indirect_bytes[indirect_offset..indirect_offset + indirect_size]
                    .copy_from_slice(bytemuck::bytes_of(args));

                let info_offset = slot * self.mesh_cull_info_stride as usize;
                let info_size = std::mem::size_of::<MeshCullInfo>();
                mesh_info_bytes[info_offset..info_offset + info_size]
                    .copy_from_slice(bytemuck::bytes_of(info));
            }

            queue.write_buffer(&self.indirect_args_buffer, 0, &indirect_bytes);
            queue.write_buffer(&self.mesh_cull_info_buffer, 0, &mesh_info_bytes);
        }

        queue.write_buffer(&self.cull_globals_buffer, 0, bytemuck::bytes_of(globals));
    }

    /// rebuilds the hi-z pyramid from whatever `depth_view` currently holds (at this
    /// point in the frame, that's still last frame's depth: the clear for this frame's
    /// depth pass hasn't been recorded yet).
    ///
    /// unused for now: occlusion culling (the only consumer) is disabled pending a
    /// redo, see cull.wgsl. kept so re-enabling it later is a small diff
    #[allow(dead_code)]
    pub(crate) fn record_hiz_build(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        size: PhysicalSize<u32>,
    ) {
        let workgroups_for = |n: u32| n.div_ceil(8);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hi-z copy pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.hiz_copy_pipeline);
            pass.set_bind_group(0, &self.hiz_copy_bind_group, &[]);
            pass.dispatch_workgroups(
                workgroups_for(size.width.max(1)),
                workgroups_for(size.height.max(1)),
                1,
            );
        }

        let mut mip_size = (size.width.max(1), size.height.max(1));
        for bind_group in &self.hiz_downsample_bind_groups {
            mip_size = ((mip_size.0 / 2).max(1), (mip_size.1 / 2).max(1));

            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hi-z downsample pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.hiz_downsample_pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.dispatch_workgroups(workgroups_for(mip_size.0), workgroups_for(mip_size.1), 1);
        }
    }

    /// one cull.wgsl dispatch per mesh, `meshes` in the same slot order passed to
    /// `write_frame_data` this frame
    pub(crate) fn record_cull_dispatches(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        meshes: &[(MeshCullInfo, DrawIndexedIndirectArgs)],
    ) {
        if meshes.is_empty() {
            return;
        }

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("cull pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.cull_pipeline);

        for (slot, (info, _)) in meshes.iter().enumerate() {
            let workgroups = info.instance_count.div_ceil(64);
            if workgroups == 0 {
                continue;
            }

            let indirect_offset = slot as u32 * self.indirect_args_stride;
            let info_offset = slot as u32 * self.mesh_cull_info_stride;
            pass.set_bind_group(0, &self.cull_bind_group, &[indirect_offset, info_offset]);
            pass.dispatch_workgroups(workgroups, 1, 1);
        }
    }

    /// true while a previous frame's readback is still waiting on its `map_async` to
    /// resolve. callers should skip `record_readback_copy` until this clears
    pub(crate) fn readback_pending(&self) -> bool {
        self.readback_pending
    }

    /// copies this frame's post-cull instance counts (one u32 per mesh slot, the second
    /// field of each slot's `DrawIndexedIndirectArgs`) into the mappable readback buffer.
    /// call `start_readback_map` afterward, once this copy has actually been submitted -
    /// mapping a buffer that a not-yet-submitted encoder still writes to is a validation
    /// error, wgpu has no way to know the copy is about to make it valid to map. no-op if
    /// a previous readback hasn't resolved yet or there's nothing to copy
    pub(crate) fn record_readback_copy(&self, encoder: &mut wgpu::CommandEncoder, slot_count: u32) {
        if self.readback_pending || slot_count == 0 {
            return;
        }

        let bytes = self.indirect_args_stride as u64 * slot_count as u64;
        encoder.copy_buffer_to_buffer(&self.indirect_args_buffer, 0, &self.readback_buffer, 0, bytes);
    }

    /// starts the async map for the copy `record_readback_copy` just recorded, once its
    /// encoder has been submitted. the result shows up a frame or two later via
    /// `take_readback`, not immediately. no-op under the same conditions
    /// `record_readback_copy` no-ops under, since they're meant to be called as a pair
    pub(crate) fn start_readback_map(&mut self, slot_count: u32) {
        if self.readback_pending || slot_count == 0 {
            return;
        }

        let bytes = self.indirect_args_stride as u64 * slot_count as u64;
        self.readback_pending = true;
        let buffer = self.readback_buffer.clone();
        let result = self.readback_result.clone();
        let stride = self.indirect_args_stride;
        self.readback_buffer
            .slice(0..bytes)
            .map_async(wgpu::MapMode::Read, move |map_result| {
                if map_result.is_err() {
                    return;
                }
                let data = buffer.slice(0..bytes).get_mapped_range();
                let counts = (0..slot_count)
                    .map(|slot| {
                        // instance_count is DrawIndexedIndirectArgs's second u32 field
                        let offset = (slot * stride) as usize + 4;
                        bytemuck::pod_read_unaligned::<u32>(&data[offset..offset + 4])
                    })
                    .collect();
                drop(data);
                buffer.unmap();
                *result.lock().unwrap() = Some(counts);
            });
    }

    /// drains this frame's completed readback, if one landed since the last call.
    /// returned counts are in the same slot order as the `meshes` slice passed to
    /// `record_readback`'s matching `write_frame_data`/dispatch call
    pub(crate) fn take_readback(&mut self) -> Option<Vec<u32>> {
        let counts = self.readback_result.lock().unwrap().take();
        if counts.is_some() {
            self.readback_pending = false;
        }
        counts
    }

    pub(crate) fn indirect_args_buffer(&self) -> &wgpu::Buffer {
        &self.indirect_args_buffer
    }

    pub(crate) fn indirect_args_stride(&self) -> u32 {
        self.indirect_args_stride
    }

    pub(crate) fn hiz_mip_count(&self) -> u32 {
        self.hiz_mip_views.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_points_gives_zero_sphere() {
        let (center, radius) = compute_bounding_sphere(&[]);
        assert_eq!(center, Vec3::ZERO);
        assert_eq!(radius, 0.0);
    }

    #[test]
    fn unit_cube_sphere_centered_at_origin() {
        let h = 0.5;
        let points = vec![
            Vec3::new(-h, -h, -h),
            Vec3::new(h, -h, -h),
            Vec3::new(-h, h, -h),
            Vec3::new(h, h, -h),
            Vec3::new(-h, -h, h),
            Vec3::new(h, -h, h),
            Vec3::new(-h, h, h),
            Vec3::new(h, h, h),
        ];
        let (center, radius) = compute_bounding_sphere(&points);
        assert!(center.abs_diff_eq(Vec3::ZERO, 1e-6));
        // half-diagonal of a unit cube
        assert!((radius - (0.75_f32).sqrt()).abs() < 1e-6);
    }

    #[test]
    fn offset_points_get_offset_center() {
        let points = vec![Vec3::new(10.0, 0.0, 0.0), Vec3::new(12.0, 0.0, 0.0)];
        let (center, radius) = compute_bounding_sphere(&points);
        assert!(center.abs_diff_eq(Vec3::new(11.0, 0.0, 0.0), 1e-6));
        assert!((radius - 1.0).abs() < 1e-6);
    }

    #[test]
    fn draw_indexed_indirect_args_is_20_bytes() {
        assert_eq!(std::mem::size_of::<DrawIndexedIndirectArgs>(), 20);
    }

    #[test]
    fn frustum_planes_finite_for_identity() {
        let planes = extract_frustum_planes(glam::Mat4::IDENTITY);
        for plane in planes {
            for v in plane {
                assert!(v.is_finite());
            }
        }
    }
}
