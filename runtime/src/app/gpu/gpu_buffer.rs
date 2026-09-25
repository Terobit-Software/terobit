use glam::Mat4;
use wgpu::util::DeviceExt;

pub(crate) struct GpuBuffer {
    number: u32,
    buffer: wgpu::Buffer,
}

impl GpuBuffer {
    pub(crate) fn builder<'a>() -> GpuBufferBuilder<'a> {
        GpuBufferBuilder::new()
    }

    pub(crate) fn get_buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub(crate) fn get_entry<'a>(&'a self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: self.number,
            resource: self.buffer.as_entire_binding(),
        }
    }
}

pub(crate) struct GpuBufferBuilder<'a> {
    number: u32,
    name: Option<&'a str>,
    init_contents: Option<&'a [u8]>,
    usage: wgpu::BufferUsages,
    device: Option<&'a wgpu::Device>,
}

impl<'a> GpuBufferBuilder<'a> {
    pub(crate) fn new() -> Self {
        Self {
            number: 0,
            name: None,
            init_contents: None,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            device: None,
        }
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

        GpuBuffer {
            number: self.number,
            buffer,
        }
    }
}
