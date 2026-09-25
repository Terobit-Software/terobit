use crate::app::gpu::GpuBuffer;
use tracing::error;

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
}

pub(crate) struct GpuBindGroupBuilder<'a> {
    name: Option<&'static str>,
    device: Option<&'a wgpu::Device>,
    buffers: Vec<GpuBuffer>,
}

impl<'a> GpuBindGroupBuilder<'a> {
    pub(crate) fn new() -> Self {
        Self {
            name: None,
            device: None,
            buffers: Vec::new(),
        }
    }

    pub(crate) fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
    }

    pub(crate) fn device(mut self, device: &'a wgpu::Device) -> Self {
        self.device = Some(device);
        self
    }

    fn build_layout(&self) -> wgpu::BindGroupLayout {
        self.device
            .expect("set device before building layout on gpu group")
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: self.name,
                entries: &[],
            })
    }

    pub(crate) fn build(self) -> GpuBindGroup {
        let mut bind_group = GpuBindGroup::default();

        match self.device {
            Some(device) => {
                let layout = self.build_layout();
                bind_group.group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: self.name,
                    layout: &layout,
                    entries: &[],
                }));
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
