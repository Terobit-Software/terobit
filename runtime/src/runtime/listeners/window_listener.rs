use crate::handle::RuntimeHandle;

pub trait WindowListener {
    fn on_window_resize(&self, width: u32, height: u32, handle: &RuntimeHandle) {}
    fn on_window_close(&self, handle: &RuntimeHandle) {}
    fn on_window_focus(&self, handle: &RuntimeHandle) {}
    fn on_window_create(&self, handle: &RuntimeHandle) {}
    fn on_window_minimize(&self, handle: &RuntimeHandle) {}
    fn on_window_maximize(&self, handle: &RuntimeHandle) {}
    fn priority(&self) -> u32;
}
