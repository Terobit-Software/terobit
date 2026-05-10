use winit::{event_loop::{ControlFlow, EventLoop}, keyboard::KeyCode};

use crate::app::{App, AppState, OnFrameHandler};

pub struct Runtime {
    app: App,
    event_loop: Option<EventLoop<()>>,
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

    pub fn on_frame<F: Fn(RuntimeContext, f32) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_frame = Some(Box::new(func));
    }

    pub fn on_start<F: Fn(RuntimeContext) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_start = Some(Box::new(func));
    }

    pub fn on_key<F: Fn(RuntimeContext, KeyCode) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_key = Some(Box::new(func));
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RuntimeContext<'a> {
    pub(crate) state: &'a mut AppState,
}

impl RuntimeContext<'_> {
    pub fn create_scene(&mut self, name: &str) {
        self.state.create_scene(String::from(name));
    }
}