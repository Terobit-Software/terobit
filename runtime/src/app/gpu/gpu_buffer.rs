pub(crate) struct GpuBuffer {
    number: u32,
}

impl GpuBuffer {
    pub(crate) fn builder<'a>() -> GpuBufferBuilder<'a> {
        GpuBufferBuilder::new()
    }
}

pub(crate) struct GpuBufferBuilder<'a> {
    number: u32,
    name: Option<&'a str>,
    init_contents: Option<&'a [u8]>,
    usage: wgpu::BufferUsages,
}

impl<'a> GpuBufferBuilder<'a> {
    pub(crate) fn new() -> Self {
        Self {
            number: 0,
            name: None,
            init_contents: None,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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
}
