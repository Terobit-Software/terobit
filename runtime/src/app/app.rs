use crate::app::state::AppState;
use thiserror::Error;
use tracing::{error, info};
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
            self.state = match pollster::block_on(AppState::new(event_loop)) {
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
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.run();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // continuously request redraws to drive the render loop; Fifo (vsync)
        // presentation paces us to the display's refresh rate
        if let Some(state) = &self.state {
            state.get_window().get_window().request_redraw();
        }
    }
}
