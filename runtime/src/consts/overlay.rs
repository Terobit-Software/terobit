/// Font size, in logical pixels, used to draw the debug overlay text.
pub const OVERLAY_FONT_SIZE: f32 = 10.0;

/// Distance, in logical pixels, from the top-left corner of the window to
/// where the overlay text is drawn.
pub const OVERLAY_MARGIN: f32 = OVERLAY_FONT_SIZE * 0.6;

/// Vertical gap, in logical pixels, between stacked overlay lines.
pub const OVERLAY_LINE_SPACING: f32 = OVERLAY_FONT_SIZE * 1.4;

/// Exponential moving average weight given to each new instantaneous FPS
/// sample (0-1); lower is smoother but slower to react.
pub const OVERLAY_FPS_SMOOTHING: f32 = 0.1;
