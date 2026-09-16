mod window_listener;
pub use window_listener::WindowListener;

pub(crate) struct ListenerManager {
    window_listeners: Vec<Box<dyn WindowListener>>,
}
