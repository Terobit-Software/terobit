use std::sync::Arc;

use winit::{event::KeyEvent, keyboard::KeyCode};

use crate::{RuntimeContext, app::KeyInput};

/// callback type for `Runtime::on_frame`. receives context and delta time in seconds
pub type OnFrameHandler = Box<dyn Fn(RuntimeContext, f32)>;

/// callback type for `Runtime::on_key`. arc because it needs to be cloneable
pub type OnKeyHandler = Arc<dyn Fn(RuntimeContext, KeyInput)>;

/// callback type for `Runtime::on_start`
pub type OnStartHandler = Box<dyn Fn(RuntimeContext)>;

/// callback type for `Runtime::on_mouse_motion`. receives (dx, dy) in physical pixels
pub type OnMouseMotionHandler = Arc<dyn Fn(RuntimeContext, f64, f64)>;

#[derive(Default)]
pub(crate) struct EventHandlerContainer {
    pub on_frame: Option<OnFrameHandler>,
    pub on_key: Option<OnKeyHandler>,
    pub on_start: Option<OnStartHandler>,
    pub on_mouse_motion: Option<OnMouseMotionHandler>,
}
