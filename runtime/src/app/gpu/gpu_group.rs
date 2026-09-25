use crate::app::gpu::GpuBuffer;

pub(crate) struct GpuBindGroup {
    number: u32,
    buffers: Vec<GpuBuffer>,
}
