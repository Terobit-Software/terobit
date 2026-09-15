#[cfg(test)]
mod tests {
    use runtime::consts::app::*;

    #[test]
    fn window_title_is_set() {
        assert_eq!(WINDOW_TITLE, "Terobit Client");
    }

    #[test]
    fn desired_maximum_frame_latency_is_two() {
        assert_eq!(DESIRED_MAXIMUM_FRAME_LATENCY, 2);
    }

    #[test]
    fn present_mode_is_auto_vsync() {
        assert_eq!(PRESENT_MODE, wgpu::PresentMode::AutoVsync);
    }

    #[test]
    fn skipped_frame_delay_is_sixteen_milliseconds() {
        assert_eq!(SKIPPED_FRAME_DELAY, std::time::Duration::from_millis(16));
    }
}
