mod window_listener;
pub use window_listener::*;

mod input_listener;
pub use input_listener::*;

use crate::handle::RuntimeHandle;

pub(crate) struct ListenerManager {
    window_listeners: Vec<Box<dyn WindowListener>>,
    input_listeners: Vec<Box<dyn InputListener>>,
}

impl ListenerManager {
    pub(crate) fn new() -> Self {
        Self {
            window_listeners: Vec::new(),
            input_listeners: Vec::new(),
        }
    }

    #[inline]
    fn add_window_listener(&mut self, listener: Box<dyn WindowListener>) {
        self.window_listeners.push(listener);
        self.window_listeners.sort_by_key(|l| l.priority());
    }

    fn add_input_listener(&mut self, listener: Box<dyn InputListener>) {
        self.input_listeners.push(listener);
        self.input_listeners.sort_by_key(|l| l.priority());
    }

    pub(crate) fn add_listener(&mut self, listener: ListenerValue) {
        match listener {
            ListenerValue::Window(window_listener) => self.add_window_listener(window_listener),
            ListenerValue::Input(input_listener) => self.add_input_listener(input_listener),
        }
    }

    // window listener events
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

    // input listener events
    pub(crate) fn on_key_input(&mut self, event: KeyInput, handle: &RuntimeHandle) {
        for listener in &mut self.input_listeners {
            listener.on_key_input(event, handle);
        }
    }
    pub(crate) fn on_mouse_input(&mut self, event: MouseInput, handle: &RuntimeHandle) {
        for listener in &mut self.input_listeners {
            listener.on_mouse_input(event, handle);
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
    Input(Box<dyn InputListener>),
}

impl ListenerValue {
    pub(crate) fn name(&self) -> &str {
        match self {
            ListenerValue::Window(listener) => listener.name(),
            ListenerValue::Input(listener) => listener.name(),
        }
    }
}
