use crate::app::state::AppState;
use crate::runtime::handle::RuntimeCommand;
use crate::runtime::listeners::ListenerValue;
use crate::runtime::routines::RuntimeRoutine;
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
    // routines registered before the state exists, drained into it on resume
    pending_routines: Vec<Box<dyn RuntimeRoutine>>,
    // listeners registered before the state exists, drained into it on resume
    pending_listeners: Vec<ListenerValue>,
}

impl RuntimeApp {
    pub fn new() -> Self {
        Self {
            state: None,
            pending_routines: Vec::new(),
            pending_listeners: Vec::new(),
        }
    }

    /// Queues a routine, installing it immediately if the state already exists.
    pub(crate) fn queue_routine(&mut self, routine: Box<dyn RuntimeRoutine>) {
        match &mut self.state {
            Some(state) => state.add_routine(routine),
            None => self.pending_routines.push(routine),
        }
    }

    /// Queues a listener, installing it immediately if the state already exists.
    pub(crate) fn queue_listener(&mut self, listener: ListenerValue) {
        match &mut self.state {
            Some(state) => state.add_listener(listener),
            None => self.pending_listeners.push(listener),
        }
    }
}

impl ApplicationHandler for RuntimeApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            self.state = match pollster::block_on(AppState::new(event_loop)) {
                Ok(mut state) => {
                    for routine in self.pending_routines.drain(..) {
                        state.add_routine(routine);
                    }
                    for listener in self.pending_listeners.drain(..) {
                        state.add_listener(listener);
                    }
                    state.notify_window_created();
                    Some(state)
                }
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
                if let Some(state) = &mut self.state {
                    state.notify_window_close();
                }
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(new_size);
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(state) = &mut self.state
                    && focused
                {
                    state.notify_window_focus();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    for command in state.run() {
                        match command {
                            RuntimeCommand::Terminate => {
                                info!("Terminate command received, exiting event loop");
                                event_loop.exit();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // continuously request redraws to drive the render loop; Fifo (vsync)
        // presentation paces us to the display's refresh rate
        if let Some(state) = &self.state
            && !_event_loop.exiting()
        {
            state.get_window().get_window().request_redraw();
        }
    }
}

impl RuntimeApp {
    #[inline]
    pub(crate) fn get_state(&self) -> Option<&AppState> {
        self.state.as_ref()
    }

    #[inline]
    pub(crate) fn get_state_mut(&mut self) -> Option<&mut AppState> {
        self.state.as_mut()
    }
}
