use crate::app::state::AppState;
use thiserror::Error;
use tracing::error;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

/// The main application struct.
pub(crate) struct RuntimeApp {
    // none until resumed hands us an active event loop
    state: Option<AppState>,
}

impl RuntimeApp {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler for RuntimeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            self.state = match AppState::new(event_loop) {
                Ok(state) => Some(state),
                Err(err) => {
                    error!("failed to create app state: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Some(state) = &self.state {
                    state.run();
                }
            }
            _ => {}
        }
    }
}
