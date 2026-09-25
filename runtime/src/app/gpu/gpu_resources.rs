struct GpuResources {
    groups: Vec<GpuBindGroup>,
}

struct GpuBindGroup {
    number: u32,
    buffers: Vec<GpuBuffer>,
}

struct GpuBuffer {
    number: u32,
}

