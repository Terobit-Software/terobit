#[cfg(test)]
mod tests {
    use runtime::consts::overlay::*;

    #[test]
    fn margin_is_derived_from_font_size() {
        assert_eq!(OVERLAY_MARGIN, OVERLAY_FONT_SIZE * 0.6);
    }

    #[test]
    fn line_spacing_is_derived_from_font_size() {
        assert_eq!(OVERLAY_LINE_SPACING, OVERLAY_FONT_SIZE * 1.4);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn fps_smoothing_is_a_valid_weight() {
        assert!(OVERLAY_FPS_SMOOTHING > 0.0 && OVERLAY_FPS_SMOOTHING < 1.0);
    }
}
