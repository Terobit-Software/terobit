pub trait WindowListener {
    fn on_window_resize(&self, width: u32, height: u32) {}
    fn on_window_close(&self) {}
    fn on_window_focus(&self) {}
    fn on_window_create(&self) {}
    fn on_window_minimize(&self) {}
    fn on_window_maximize(&self) {}
}
