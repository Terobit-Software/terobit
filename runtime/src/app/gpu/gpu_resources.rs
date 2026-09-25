use crate::AppState;
use crate::app::gpu::GpuBindGroup;
use crate::app::gpu::GpuBuffer;

pub(crate) struct GpuResources {
    groups: Vec<GpuBindGroup>,
}

impl GpuResources {
    pub(crate) fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub(crate) fn get_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.groups
            .iter()
            .map(|group| group.get_layout())
            .collect::<Vec<_>>()
    }

    pub(crate) fn get_groups(&self) -> &[GpuBindGroup] {
        &self.groups
    }

    pub(crate) fn add_group(&mut self, group: GpuBindGroup) {
        self.groups.push(group);
    }
}
