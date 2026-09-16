mod window_listener;
pub use window_listener::WindowListener;

pub(crate) struct ListenerManager {
    window_listeners: Vec<Box<dyn WindowListener>>,
}

impl ListenerManager {
    pub(crate) fn new() -> Self {
        Self {
            window_listeners: Vec::new(),
        }
    }

    pub(crate) fn add_window_listener(&mut self, listener: Box<dyn WindowListener>) {
        self.window_listeners.push(listener);
        self.window_listeners.sort_by_key(|l| l.priority());
    }
}
