use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("io error: {source} ({path})")]
pub struct RariIoError {
    pub path: PathBuf,
    pub source: std::io::Error,
}
