use std::sync::Arc;

use wgpu::rwh::HasWindowHandle;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, Window, WindowAttributes, WindowId},
};

use crate::{
    RuntimeContext,
    app::{AppState, EventHandlerContainer},
};

#[derive(Default)]
pub struct App {
    state: Option<AppState>,
    is_fullscreen: bool,
    handlers: EventHandlerContainer,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // create the window
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::with_visible(
                    Window::default_attributes(),
                    false,
                ))
                .unwrap(),
        );

        // create the app state
        let state = pollster::block_on(AppState::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        // slap dat state in
        self.state = Some(state);

        // make the window visible
        window.set_visible(true);

        // set the title
        window.set_title("Terobit Player");

        // stop them from resizing to be too small lol
        window.set_min_inner_size(Some(winit::dpi::LogicalSize::new(200.0, 100.0)));

        // make my window your focus
        window.focus_window();

        // redraw for the first time
        window.request_redraw();

        if let Some(handler) = &self.handlers.on_start {
            let ctx = RuntimeContext {
                state: self.state.as_mut().unwrap(),
            };
            handler(ctx)
        }
    }

    /// when something happens in the window we should probably handle it tbh
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("window said please close me");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(handler) = &self.handlers.on_frame {
                    let ctx = RuntimeContext { state: &mut *state };
                    handler(ctx, 0.0)
                }

                // does what it says lol
                state.render();
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return; // skip resizing if its 0
                }

                // resize the screen
                state.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::F11),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if key_state.is_pressed() {
                    self.is_fullscreen = !self.is_fullscreen;

                    if self.is_fullscreen {
                        state
                            .get_window()
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        state.get_window().set_fullscreen(None);
                    }
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.state.as_ref() {
            state.get_window().request_redraw();
        }
    }
}

impl App {
    pub fn get_state(&self) -> &Option<AppState> {
        &self.state
    }

    pub fn get_state_mut(&mut self) -> &mut Option<AppState> {
        &mut self.state
    }

    pub fn get_handlers(&self) -> &EventHandlerContainer {
        &self.handlers
    }

    pub fn get_handlers_mut(&mut self) -> &mut EventHandlerContainer {
        &mut self.handlers
    }
}
