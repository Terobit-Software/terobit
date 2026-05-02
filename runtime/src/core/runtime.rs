use winit::{event_loop::{ControlFlow, EventLoop}};

use crate::app::App;

pub struct Runtime {
    app: App,
    event_loop: Option<EventLoop<()>>
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            app: App::default(),
            event_loop: Some(EventLoop::new().unwrap()),
        }
    }

    pub fn start(&mut self) {
        let event_loop = self.event_loop.take().expect("EventLoop already consumed");

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self.app).unwrap();
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}