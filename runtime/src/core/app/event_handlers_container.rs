use std::sync::Arc;

use winit::{event::KeyEvent, keyboard::KeyCode};

use crate::{RuntimeContext, app::KeyInput};

pub type OnFrameHandler = Box<dyn Fn(RuntimeContext, f32)>;
pub type OnKeyHandler = Arc<dyn Fn(RuntimeContext, KeyInput)>;
pub type OnStartHandler = Box<dyn Fn(RuntimeContext)>;

#[derive(Default)]
pub(crate) struct EventHandlerContainer {
    pub on_frame: Option<OnFrameHandler>,
    pub on_key: Option<OnKeyHandler>,
    pub on_start: Option<OnStartHandler>,
}
