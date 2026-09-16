use crate::handle::RuntimeHandle;
use crate::listeners::Listener;

pub enum KeyInput {
    KeyPressed(char),
    KeyRepeat(char),
    KeyReleased(char),
}

pub enum MouseInputType {
    Down,
    Up,
    Drag,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub struct MouseInput {
    pub input_type: MouseInputType,
    pub x: f32,
    pub y: f32,
    pub button: MouseButton,
}

pub trait InputListener: Listener {
    fn on_key_input(&mut self, input: KeyInput);
    fn on_mouse_input(&mut self, input: MouseInput);
}
