use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AssetError {
    #[error("failed to read file")]
    FailedToReadFile(#[from] std::io::Error),
}
