use std::sync::Arc;

use winit::{
    event::KeyEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
};

use crate::{
    app::{App, AppState, KeyInput, OnFrameHandler},
    assets::MeshLoadOptions,
    ecs::{Mesh, SceneHandle},
    profiling::ProfilerScope,
};

/// the main entry point for the engine. create one, register callbacks, then call `start()`
pub struct Runtime {
    app: App,
    event_loop: Option<EventLoop<()>>,
}

impl Runtime {
    /// creates a new runtime and initializes logging
    pub fn new() -> Self {
        env_logger::init();

        Self {
            app: App::default(),
            event_loop: Some(EventLoop::new().unwrap()),
        }
    }

    /// starts the event loop. blocks until the window is closed
    pub fn start(&mut self) {
        let event_loop = self.event_loop.take().expect("EventLoop already consumed");

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self.app).unwrap();
    }

    /// registers a callback fired every frame. delta time is in seconds
    pub fn on_frame<F: Fn(RuntimeContext, f32) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_frame = Some(Box::new(func));
    }

    /// registers a callback fired once after the window and gpu are ready
    pub fn on_start<F: Fn(RuntimeContext) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_start = Some(Box::new(func));
    }

    /// registers a callback fired on every keyboard event
    pub fn on_key<F: Fn(RuntimeContext, KeyInput) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_key = Some(Arc::new(func));
    }

    /// registers a callback fired on raw mouse motion. dx/dy are physical pixel deltas,
    /// unaffected by cursor position or os acceleration, ideal for fps look
    pub fn on_mouse_motion<F: Fn(RuntimeContext, f64, f64) + 'static>(&mut self, func: F) {
        self.app.get_handlers_mut().on_mouse_motion = Some(Arc::new(func));
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

/// context passed into all callbacks. gives access to scenes and gpu state
pub struct RuntimeContext<'a> {
    pub state: &'a mut AppState,
}

impl<'a> RuntimeContext<'a> {
    /// creates a new named scene and returns a handle to it.
    /// the first scene created automatically becomes the active render target
    pub fn create_scene(&mut self, name: &str) -> SceneHandle<'_> {
        let n = String::from(name);
        self.state.create_scene(n.clone());

        SceneHandle {
            scene: self.state.get_scene_mut(n).unwrap(),
        }
    }

    /// returns a handle to an existing scene by name, or none if it doesn't exist
    pub fn get_scene(&mut self, name: &str) -> Option<SceneHandle<'_>> {
        self.state
            .get_scene_mut(String::from(name))
            .map(|scene| SceneHandle { scene })
    }

    /// locks the cursor to the window center and hides it (use for fps mouse look)
    pub fn grab_cursor(&self) {
        let window = self.state.get_window();
        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
        window.set_cursor_visible(false);
    }

    /// releases cursor grab and makes cursor visible again
    pub fn release_cursor(&self) {
        let window = self.state.get_window();
        let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
        window.set_cursor_visible(true);
    }

    /// loads a mesh from an obj file, uploads it to the gpu, and returns a `Mesh` component
    /// ready to spawn. calling with the same path twice reuses the cached gpu data
    pub fn load_mesh(&mut self, path: &str) -> anyhow::Result<Mesh> {
        self.load_mesh_with(path, MeshLoadOptions::default())
    }

    /// same as `load_mesh` but with explicit load options, e.g. meshopt simplification
    /// for meshes way denser than they need to be. the cache keys on (path, options)
    /// so one file can be loaded at several detail levels
    pub fn load_mesh_with(&mut self, path: &str, options: MeshLoadOptions) -> anyhow::Result<Mesh> {
        let id = self
            .state
            .load_mesh_from_path(std::path::Path::new(path), options)?;
        Ok(Mesh::with_source_id(id))
    }

    /// times a labeled section of game code, same profiler the engine uses for
    /// physics/rendering. shows up in the f7+p hud alongside the built-in sections
    pub fn profile_scope(&mut self, label: &str) -> ProfilerScope<'_> {
        self.state.profiler_mut().scope(label)
    }
}
