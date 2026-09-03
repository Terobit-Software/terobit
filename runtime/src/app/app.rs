use crate::app::state::AppState;

/// The main application struct.
pub struct App {
    state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }

    pub fn run(&self) {
        self.state.run();
    }
}
