use std::sync::Arc;

use winit::{
    event::KeyEvent, event_loop::{ControlFlow, EventLoop}, keyboard::KeyCode
};

use crate::{
    app::{App, AppState, KeyInput, OnFrameHandler},
    ecs::SceneHandle,
};

pub struct Runtime {
    app: App,
    event_loop: Option<EventLoop<()>>,
}

impl Runtime {
    pub fn new() -> Self {
        env_logger::init();

        Self {
            app: App::default(),
            event_loop: Some(EventLoop::new().unwrap()),
        }
    }

    pub fn start(&mut self) {
        let event_loop = self.event_loop.take().expect("EventLoop already consumed");

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self.app).unwrap();
    }

    pub fn on_frame<F: Fn(RuntimeContext, f32) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_frame = Some(Box::new(func));
    }

    pub fn on_start<F: Fn(RuntimeContext) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_start = Some(Box::new(func));
    }

    pub fn on_key<F: Fn(RuntimeContext, KeyInput) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_key = Some(Arc::new(func));
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RuntimeContext<'a> {
    pub state: &'a mut AppState,
}

impl<'a> RuntimeContext<'a> {
    pub fn create_scene(&mut self, name: &str) -> SceneHandle<'_> {
        let n = String::from(name);
        self.state.create_scene(n.clone());

        SceneHandle {
            scene: self.state.get_scene_mut(n).unwrap(),
        }
    }

    pub fn get_scene(&mut self, name: &str) -> Option<SceneHandle<'_>> {
        self.state
            .get_scene_mut(String::from(name))
            .map(|scene| SceneHandle { scene })
    }
}