use std::path::Path;

use crate::error::DepsError;
use crate::npm::get_package;

pub fn update_mdn_data(base_path: &Path) -> Result<(), DepsError> {
    get_package("mdn-data", None, base_path)?;
    Ok(())
}
