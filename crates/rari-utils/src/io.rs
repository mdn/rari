use std::fs;
use std::path::Path;

use crate::error::RariIoError;

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, RariIoError> {
    fs::read_to_string(path.as_ref()).map_err(|e| RariIoError {
        source: e,
        path: path.as_ref().to_path_buf(),
    })
}
