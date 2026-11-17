use std::time::Duration;

/// Timing information for the current frame.
///
/// `#[repr(C)]` and `Pod` so the host can write it straight into guest memory
/// as raw bytes (24 bytes, no padding).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameInfo {
    /// Seconds since the previous frame.
    delta_secs: f64,
    /// Seconds since the script was loaded.
    elapsed_secs: f64,
    /// Number of frames since the script was loaded, starting at 0.
    frame: u64,
}

impl FrameInfo {
    /// Fetches the timing information for the current frame from the host drop spot.
    pub fn get() -> Self {
        // TODO: grab from the data that the host fills in
        Self::default()
    }

    /// Time since the previous frame.
    pub fn delta(&self) -> Duration {
        Duration::from_secs_f64(self.delta_secs)
    }

    /// Time since the previous frame, in seconds.
    pub fn delta_secs(&self) -> f64 {
        self.delta_secs
    }

    /// Time since the script was loaded.
    pub fn elapsed(&self) -> Duration {
        Duration::from_secs_f64(self.elapsed_secs)
    }

    /// Time since the script was loaded, in seconds.
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed_secs
    }

    /// Number of frames since the script was loaded, starting at 0.
    pub fn frame(&self) -> u64 {
        self.frame
    }
}
