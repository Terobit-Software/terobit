use winit::dpi::PhysicalPosition;

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
    Move,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    None,
}

pub type ScreenPosition = PhysicalPosition<f64>;
#[derive(Debug, Clone, Copy)]
pub enum MouseInput {
    Move(ScreenPosition),
    Down(ScreenPosition, MouseButton),
    Up(ScreenPosition, MouseButton),
    Drag(ScreenPosition, MouseButton),
}

pub trait InputListener: Listener {
    fn on_key_input(&mut self, input: KeyInput, _handle: &RuntimeHandle);
    fn on_mouse_input(&mut self, input: MouseInput, _handle: &RuntimeHandle);
}
