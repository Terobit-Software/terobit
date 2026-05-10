use winit::keyboard::KeyCode;

use crate::RuntimeContext;

pub type OnFrameHandler = Box<dyn Fn(RuntimeContext, f32)>;
pub type OnKeyHandler = Box<dyn Fn(RuntimeContext, KeyCode)>;
pub type OnStartHandler = Box<dyn Fn(RuntimeContext)>;

#[derive(Default)]
pub struct EventHandlerContainer {
    pub on_frame: Option<OnFrameHandler>,
    pub on_key: Option<OnKeyHandler>,
    pub on_start: Option<OnStartHandler>,
}