use crate::handle::RuntimeHandle;
use crate::listeners::Listener;

pub trait WindowListener: Listener {
    fn on_window_resize(&self, _width: u32, _height: u32, _handle: &RuntimeHandle) {}
    fn on_window_close(&self, _handle: &RuntimeHandle) {}
    fn on_window_focus(&self, _handle: &RuntimeHandle) {}
    fn on_window_create(&self, _handle: &RuntimeHandle) {}
}
