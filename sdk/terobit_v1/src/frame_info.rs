use core::cell::Cell;
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
        FRAME_INFO.with(Cell::get)
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

// check the stated sizes are the same
const _: () =
    assert!(size_of::<FrameInfo>() == size_of::<terobit_v1_abi::frame_info::FrameInfo>() as usize);

thread_local! {
    /// Drop spot the host writes the current frame's [`FrameInfo`] into.
    static FRAME_INFO: Cell<FrameInfo> = const {
        Cell::new(FrameInfo { delta_secs: 0.0, elapsed_secs: 0.0, frame: 0 })
    };
}

// Export the host uses to find the drop spot.
// Name matches `terobit_v1_abi::endpoints::FRAME_INFO_PTR_ENDPOINT`.

#[unsafe(no_mangle)]
pub extern "C" fn _terobit_v1_frame_info_ptr() -> i32 {
    FRAME_INFO.with(|info| info.as_ptr() as i32)
}
