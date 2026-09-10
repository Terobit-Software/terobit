use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("failed to read file")]
    FailedToReadFile(#[from] std::io::Error),
}
