use crate::app::gpu::GpuBindGroup;
use crate::app::gpu::GpuBuffer;

pub(crate) struct GpuResources {
    groups: Vec<GpuBindGroup>,
}
