#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameInfo {
    /// Seconds since the previous frame.
    pub delta_secs: f64,
    /// Seconds since the script was loaded.
    pub elapsed_secs: f64,
    /// Number of frames since the script was loaded, starting at 0.
    pub frame: u64,
}

/// Byte length of [`FrameInfo`].
pub const FRAME_INFO_LEN: usize = core::mem::size_of::<FrameInfo>();

const _: () = assert!(FRAME_INFO_LEN == 24);
