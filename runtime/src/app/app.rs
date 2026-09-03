use crate::app::state::AppState;

/// The main application struct.
pub(crate) struct RuntimeApp {
    // none until we can build AppState with a real event loop
    state: Option<AppState>,
}

impl RuntimeApp {
    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn run(&self) {
        if let Some(state) = &self.state {
            state.run();
        }
    }
}
