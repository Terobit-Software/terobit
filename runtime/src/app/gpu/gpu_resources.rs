use crate::AppState;
use crate::app::gpu::GpuBindGroup;
use crate::app::gpu::GpuBuffer;

pub(crate) struct GpuResources<'a> {
    groups: Vec<GpuBindGroup>,
    state: &'a AppState,
}

impl<'a> GpuResources<'a> {
    pub(crate) fn new(state: &'a AppState) -> Self {
        Self {
            groups: Vec::new(),
            state,
        }
    }

    pub(crate) fn get_groups(&self) -> &[GpuBindGroup] {
        &self.groups
    }

    pub(crate) fn add_group(&mut self, group: GpuBindGroup) {
        self.groups.push(group);
    }
}
