use crate::app::gpu::GpuBuffer;
use tracing::error;
use wgpu::{BindGroupLayoutEntry, ShaderStages, wgc::device};

#[derive(Default)]
pub(crate) struct GpuBindGroup {
    number: u32,
    buffers: Vec<GpuBuffer>,
    group: Option<wgpu::BindGroup>,
    layout: Option<wgpu::BindGroupLayout>,
    ready: bool,
}

impl GpuBindGroup {
    pub(crate) fn builder<'a>() -> GpuBindGroupBuilder<'a> {
        GpuBindGroupBuilder::new()
    }

    pub(crate) fn get_layout(&self) -> &wgpu::BindGroupLayout {
        self.layout
            .as_ref()
            .expect("gpu bind group incorrectly built, no layout")
    }

    pub(crate) fn get_group(&self) -> &wgpu::BindGroup {
        self.group
            .as_ref()
            .expect("gpu bind group incorrectly built, no group")
    }

    pub(crate) fn get_buffers(&self) -> &[GpuBuffer] {
        &self.buffers
    }
}

pub(crate) struct GpuBindGroupBuilder<'a> {
    name: Option<&'a str>,
    device: Option<&'a wgpu::Device>,
    buffers: Vec<GpuBuffer>,
    number: Option<u32>,
    visibility: Option<ShaderStages>,
}

impl<'a> GpuBindGroupBuilder<'a> {
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            device: None,
            buffers: Vec::new(),
            number: None,
            visibility: None,
        }
    }

    pub(crate) fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    pub(crate) fn device(mut self, device: &'a wgpu::Device) -> Self {
        self.device = Some(device);
        self
    }

    pub(crate) fn number(mut self, number: u32) -> Self {
        self.number = Some(number);
        self
    }

    pub(crate) fn visibility(mut self, visibility: wgpu::ShaderStages) -> Self {
        self.visibility = Some(visibility);
        self
    }

    pub(crate) fn buffer(mut self, buffer: GpuBuffer) -> Self {
        self.buffers.push(buffer);
        self
    }

    fn build_layout(&self) -> wgpu::BindGroupLayout {
        self.device
            .expect("set device before building layout on gpu group")
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.name,
                entries: &[BindGroupLayoutEntry {
                    binding: self
                        .number
                        .expect("set bind group number before building group layout"),
                    visibility: self
                        .visibility
                        .expect("set visibilty group before building group layout"),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    pub(crate) fn build(self) -> GpuBindGroup {
        let mut bind_group = GpuBindGroup::default();

        match self.device {
            Some(device) => {
                let layout = self.build_layout();
                bind_group.group = Some(
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: self.name,
                        layout: &layout,
                        entries: &self
                            .buffers
                            .iter()
                            .map(|buf| buf.get_entry())
                            .collect::<Vec<_>>(), // this is amazing
                    }),
                );
                bind_group.layout = Some(layout);
                bind_group.ready = true;

                bind_group
            }
            None => {
                error!("set device before running build on gpu bind group");
                panic!("device is not available for gpu bind group builder");
            }
        }
    }
}
