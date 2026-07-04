use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

pub use winit::{keyboard::{Key, KeyCode, NamedKey, PhysicalKey}};

use crate::{
    RuntimeContext,
    app::{AppState, EventHandlerContainer},
};

/// keyboard event passed to `on_key` callbacks
pub struct KeyInput {
    /// layout-aware logical key (e.g. 'A', 'Enter')
    pub key: winit::keyboard::Key,
    /// position-based physical key, layout-independent
    pub physical_key: winit::keyboard::PhysicalKey,
    /// pressed or released
    pub state: winit::event::ElementState,
    /// true if this is a key-repeat event (key held down)
    pub repeat: bool,
    /// the text produced by this keypress, if any
    pub text: Option<winit::keyboard::SmolStr>,
}

#[derive(Default)]
pub(crate) struct App {
    state: Option<AppState>,
    handlers: EventHandlerContainer,
    /// tracked here (not just in client key handlers) so the f7+p profiler hud
    /// toggle always works regardless of what the game's `on_key` callback does
    f7_held: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::with_visible(
                    Window::default_attributes(),
                    false,
                ))
                .unwrap(),
        );

        let state = pollster::block_on(AppState::new(
            event_loop.owned_display_handle(),
            window.clone(),
        ));
        self.state = Some(state);

        window.set_visible(true);
        window.set_title("Terobit Player");
        // stop them from resizing to be too small
        window.set_min_inner_size(Some(winit::dpi::LogicalSize::new(200.0, 100.0)));
        window.focus_window();
        window.request_redraw();

        if let Some(handler) = &self.handlers.on_start {
            let ctx = RuntimeContext {
                state: self.state.as_mut().unwrap(),
            };
            handler(ctx)
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("window said please close me");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let dt = state.tick_delta();
                state.profiler_mut().start("frame");

                state.profiler_mut().start("on_frame");
                if let Some(handler) = &self.handlers.on_frame {
                    let ctx = RuntimeContext { state: &mut *state };
                    handler(ctx, dt)
                }
                state.profiler_mut().stop("on_frame");

                state.profiler_mut().start("physics");
                state.step_physics(dt);
                state.profiler_mut().stop("physics");

                state.profiler_mut().start("render");
                state.render();
                state.profiler_mut().stop("render");

                state.profiler_mut().stop("frame");
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    // a zero-size surface is invalid to configure, wgpu will panic on it
                    return;
                }

                state.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        logical_key,
                        state,
                        repeat,
                        text,
                        location,
                        ..
                    },
                ..
            } => {
                // f7+p toggles the profiler hud, tracked here so it always works
                // regardless of what the game's own on_key callback does
                if physical_key == PhysicalKey::Code(KeyCode::F7) {
                    self.f7_held = state.is_pressed();
                } else if physical_key == PhysicalKey::Code(KeyCode::KeyP)
                    && state.is_pressed()
                    && !repeat
                    && self.f7_held
                {
                    self.state.as_mut().unwrap().toggle_profiler_hud();
                }

                let handler = self.get_handlers().on_key.clone();
                if let Some(handler) = handler {
                    let input = KeyInput {
                        key: logical_key,
                        physical_key,
                        state,
                        repeat,
                        text,
                    };
                    let ctx = RuntimeContext {
                        state: self.state.as_mut().unwrap(),
                    };
                    handler(ctx, input);
                }
            }
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            let handler = self.handlers.on_mouse_motion.clone();
            if let Some(handler) = handler {
                if let Some(state) = self.state.as_mut() {
                    let ctx = RuntimeContext { state };
                    handler(ctx, dx, dy);
                }
            }
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
