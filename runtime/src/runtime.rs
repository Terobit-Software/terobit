use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeInitError {
    #[error("failed to initialize")]
    InitFailed,
}

pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn init(&self) -> Result<(), RuntimeInitError> {
        Ok(())
    }
}
