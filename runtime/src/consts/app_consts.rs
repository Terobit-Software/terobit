/// Title applied to the main window when it is created.
pub const WINDOW_TITLE: &str = "Terobit Client";

/// Maximum number of frames the presentation engine is allowed to queue ahead.
pub const DESIRED_MAXIMUM_FRAME_LATENCY: u32 = 2;

/// Presentation mode used when configuring the window surface.
pub const PRESENT_MODE: wgpu::PresentMode = wgpu::PresentMode::AutoVsync;
