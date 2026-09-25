use std::num::NonZero;

use glam::Mat4;
use tracing::info;
use wgpu::util::DeviceExt;

pub(crate) enum BufferType {
    Uniform,
    BufferReadOnly,
    BufferReadable,
}

fn buffer_type_to_wgpu(ty: &BufferType) -> wgpu::BufferBindingType {
    match ty {
        BufferType::Uniform => wgpu::BufferBindingType::Uniform,
        BufferType::BufferReadable => wgpu::BufferBindingType::Storage { read_only: false },
        BufferType::BufferReadOnly => wgpu::BufferBindingType::Storage { read_only: true },
    }
}

pub(crate) struct GpuBuffer {
    number: u32,
    buffer: wgpu::Buffer,
    visibility: wgpu::ShaderStages,
    buffer_type: BufferType,
}

impl GpuBuffer {
    pub(crate) fn builder<'a>() -> GpuBufferBuilder<'a> {
        GpuBufferBuilder::new()
    }

    pub(crate) fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn get_bind_group_entry<'a>(&'a self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: self.number,
            resource: self.buffer.as_entire_binding(),
        }
    }

    pub(crate) fn get_layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.number,
            visibility: self.visibility,
            count: NonZero::new(self.number + 1),
            ty: wgpu::BindingType::Buffer {
                ty: buffer_type_to_wgpu(&self.buffer_type),
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        }
    }
}

pub(crate) struct GpuBufferBuilder<'a> {
    number: u32,
    name: Option<&'a str>,
    init_contents: Option<&'a [u8]>,
    usage: wgpu::BufferUsages,
    device: Option<&'a wgpu::Device>,
    visibility: Option<wgpu::ShaderStages>,
    buffer_type: Option<BufferType>,
}

impl<'a> GpuBufferBuilder<'a> {
    pub(crate) fn new() -> Self {
        Self {
            number: 0,
            name: None,
            init_contents: None,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            device: None,
            visibility: None,
            buffer_type: None,
        }
    }

    pub(crate) fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.visibility = Some(visibility);
        self
    }

    pub(crate) fn ty(mut self, ty: BufferType) -> Self {
        self.buffer_type = Some(ty);
        self
    }

    pub(crate) fn number(mut self, number: u32) -> Self {
        self.number = number;
        self
    }

    pub(crate) fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub(crate) fn contents(mut self, contents: &'a [u8]) -> Self {
        self.init_contents = Some(contents);
        self
    }

    pub(crate) fn device(mut self, device: &'a wgpu::Device) -> Self {
        self.device = Some(device);
        self
    }

    pub(crate) fn build(self) -> GpuBuffer {
        let buffer = self
            .device
            .expect("no device, add device before building gpu buffer")
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("camera uniform buffer"),
                contents: bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        info!(
            "built gpu buffer with number {:?}, name {:?}, and usage {:?}",
            self.number, self.name, self.usage
        );

        GpuBuffer {
            number: self.number,
            buffer,
            visibility: self.visibility.unwrap_or(wgpu::ShaderStages::all()),
            buffer_type: self.buffer_type.unwrap_or(BufferType::Uniform),
        }
    }
}
