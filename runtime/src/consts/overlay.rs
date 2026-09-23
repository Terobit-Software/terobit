/// Font size, in logical pixels, used to draw the debug overlay text.
pub const OVERLAY_FONT_SIZE: f32 = 10.0;

/// Distance, in logical pixels, from the top-left corner of the window to
/// where the overlay text is drawn.
pub const OVERLAY_MARGIN: f32 = OVERLAY_FONT_SIZE * 0.6;

/// How long the overlay counts frames before refreshing its FPS readout
/// (frames counted / time elapsed over the window).
pub const OVERLAY_FPS_SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);
