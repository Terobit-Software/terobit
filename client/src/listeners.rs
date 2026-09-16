use runtime::ecs::Mesh;
use runtime::handle::RuntimeHandle;
use runtime::listeners::{InputListener, Listener, WindowListener};
use runtime::listeners::{KeyInput, MouseInput};

pub struct ClientWindowListener {}

impl WindowListener for ClientWindowListener {
    fn on_window_create(&self, _handle: &RuntimeHandle) {
        _handle.world().spawn(Mesh::new(0));
    }
}

impl Listener for ClientWindowListener {
    fn name(&self) -> &str {
        "main_window_listener"
    }
    fn priority(&self) -> u8 {
        0
    }
}

pub struct ClientInputListener {}

impl Listener for ClientInputListener {
    fn name(&self) -> &str {
        "client_input_listener"
    }
    fn priority(&self) -> u8 {
        0
    }
}

impl InputListener for ClientInputListener {
    fn on_key_input(&mut self, input: KeyInput, _handle: &RuntimeHandle) {
        println!("{:?}", input)
    }
    fn on_mouse_input(&mut self, input: MouseInput, _handle: &RuntimeHandle) {
        println!("{:?}", input)
    }
}
