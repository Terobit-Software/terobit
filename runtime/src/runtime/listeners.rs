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

    #[inline]
    pub(crate) fn add_window_listener(&mut self, listener: Box<dyn WindowListener + Listener>) {
        self.window_listeners.push(listener);
        self.window_listeners.sort_by_key(|l| l.priority());
    }

    pub(crate) fn add_listener(&mut self, listener: ListenerValue) {
        match listener {
            ListenerValue::Window(window_listener) => self.add_window_listener(window_listener),
        }
    }
}

pub trait Listener {
    fn name(&self) -> &str;
    fn priority(&self) -> u8;
}

pub enum ListenerValue {
    Window(Box<dyn Listener>),
}
