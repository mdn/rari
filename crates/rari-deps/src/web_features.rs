use std::path::Path;

use crate::error::DepsError;
use crate::npm::get_package;

pub fn update_web_features(base_path: &Path) -> Result<(), DepsError> {
    get_package("web-features", None, base_path)?;
    Ok(())
}
