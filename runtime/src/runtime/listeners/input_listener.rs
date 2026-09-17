use crate::handle::RuntimeHandle;
use crate::listeners::Listener;

#[derive(Debug, Clone, Copy)]
pub enum KeyInput {
    KeyPressed(char),
    KeyRepeat(char),
    KeyReleased(char),
}

#[derive(Debug, Clone, Copy)]
pub enum MouseInputType {
    Down,
    Up,
    Drag,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub struct MouseInput {
    pub input_type: MouseInputType,
    pub x: f32,
    pub y: f32,
    pub button: MouseButton,
}

pub trait InputListener: Listener {
    fn on_key_input(&mut self, input: KeyInput, _handle: &RuntimeHandle);
    fn on_mouse_input(&mut self, input: MouseInput, _handle: &RuntimeHandle);
}
