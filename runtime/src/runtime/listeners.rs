mod window_listener;
pub use window_listener::WindowListener;

use crate::handle::RuntimeHandle;

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
    fn add_window_listener(&mut self, listener: Box<dyn WindowListener>) {
        self.window_listeners.push(listener);
        self.window_listeners.sort_by_key(|l| l.priority());
    }

    pub(crate) fn add_listener(&mut self, listener: ListenerValue) {
        match listener {
            ListenerValue::Window(window_listener) => self.add_window_listener(window_listener),
        }
    }

    pub(crate) fn on_window_create(&self, handle: &RuntimeHandle) {
        for listener in &self.window_listeners {
            listener.on_window_create(handle);
        }
    }

    pub(crate) fn on_window_resize(&self, width: u32, height: u32, handle: &RuntimeHandle) {
        for listener in &self.window_listeners {
            listener.on_window_resize(width, height, handle);
        }
    }

    pub(crate) fn on_window_close(&self, handle: &RuntimeHandle) {
        for listener in &self.window_listeners {
            listener.on_window_close(handle);
        }
    }

    pub(crate) fn on_window_focus(&self, handle: &RuntimeHandle) {
        for listener in &self.window_listeners {
            listener.on_window_focus(handle);
        }
    }
}

/// A listener that can be registered with the runtime.
pub trait Listener {
    /// The name of the listener.
    fn name(&self) -> &str;
    /// The priority of the listener. Two same-priority listeners will run in the order they were added.
    /// A lower priority value means the listener will be run earlier.
    fn priority(&self) -> u8;
}

pub enum ListenerValue {
    Window(Box<dyn WindowListener>),
}

impl ListenerValue {
    pub(crate) fn name(&self) -> &str {
        match self {
            ListenerValue::Window(listener) => listener.name(),
        }
    }
}
