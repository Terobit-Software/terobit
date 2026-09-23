#[cfg(test)]
mod tests {
    use runtime::consts::overlay::*;

    #[test]
    fn margin_is_derived_from_font_size() {
        assert_eq!(OVERLAY_MARGIN, OVERLAY_FONT_SIZE * 0.6);
    }

    #[test]
    fn fps_sample_interval_is_nonzero() {
        assert!(!OVERLAY_FPS_SAMPLE_INTERVAL.is_zero());
    }
}
