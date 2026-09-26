use crate::app::gpu::GpuBindGroup;

pub(crate) struct GpuResources {
    render_groups: Vec<GpuBindGroup>,
    compute_groups: Vec<GpuBindGroup>,
}

impl GpuResources {
    pub(crate) fn new() -> Self {
        Self {
            render_groups: Vec::new(),
            compute_groups: Vec::new(),
        }
    }

    pub(crate) fn get_render_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.render_groups
            .iter()
            .map(|group| group.get_layout())
            .collect::<Vec<_>>()
    }

    pub(crate) fn get_compute_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.compute_groups
            .iter()
            .map(|group| group.get_layout())
            .collect::<Vec<_>>()
    }

    pub(crate) fn get_render_groups(&self) -> &[GpuBindGroup] {
        &self.render_groups
    }

    pub(crate) fn get_compute_groups(&self) -> &[GpuBindGroup] {
        &self.compute_groups
    }

    pub(crate) fn add_render_group(&mut self, group: GpuBindGroup) {
        self.render_groups.push(group);
    }

    pub(crate) fn add_compute_group(&mut self, group: GpuBindGroup) {
        self.compute_groups.push(group);
    }
}
