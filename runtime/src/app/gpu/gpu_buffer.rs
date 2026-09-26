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
            count: None,
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
    size: Option<u64>,
    existing: Option<wgpu::Buffer>,
    extra_usage: wgpu::BufferUsages,
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
            size: None,
            existing: None,
            extra_usage: wgpu::BufferUsages::empty(),
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

    pub(crate) fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub(crate) fn from_buffer(mut self, buffer: wgpu::Buffer) -> Self {
        self.existing = Some(buffer);
        self
    }

    pub(crate) fn usage(mut self, usage: wgpu::BufferUsages) -> Self {
        self.extra_usage |= usage;
        self
    }

    pub(crate) fn device(mut self, device: &'a wgpu::Device) -> Self {
        self.device = Some(device);
        self
    }

    pub(crate) fn build(self) -> GpuBuffer {
        let buffer_type = self.buffer_type.unwrap_or(BufferType::Uniform);

        if let Some(buffer) = self.existing {
            info!(
                "built gpu buffer with number {:?}, name {:?}, from existing buffer (size {}, usage {:?})",
                self.number,
                self.name,
                buffer.size(),
                buffer.usage()
            );

            return GpuBuffer {
                number: self.number,
                buffer,
                visibility: self.visibility.unwrap_or(wgpu::ShaderStages::all()),
                buffer_type,
            };
        }

        let device = self
            .device
            .expect("no device, add device before building gpu buffer");
        let usage = match buffer_type {
            BufferType::Uniform => wgpu::BufferUsages::UNIFORM,
            BufferType::BufferReadOnly | BufferType::BufferReadable => wgpu::BufferUsages::STORAGE,
        } | wgpu::BufferUsages::COPY_DST
            | self.extra_usage;

        let buffer = match (self.init_contents, self.size) {
            (Some(contents), _) => device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: self.name,
                contents,
                usage,
            }),
            (None, Some(size)) => device.create_buffer(&wgpu::BufferDescriptor {
                label: self.name,
                size,
                usage,
                mapped_at_creation: false,
            }),
            (None, None) => panic!("set contents or size before building gpu buffer"),
        };

        info!(
            "built gpu buffer with number {:?}, name {:?}, size {}, and usage {:?}",
            self.number,
            self.name,
            buffer.size(),
            usage
        );

        GpuBuffer {
            number: self.number,
            buffer,
            visibility: self.visibility.unwrap_or(wgpu::ShaderStages::all()),
            buffer_type,
        }
    }
}
