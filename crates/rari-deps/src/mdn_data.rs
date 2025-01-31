use std::path::Path;

use rari_types::globals::deps;

use crate::error::DepsError;
use crate::npm::get_package;

pub fn update_mdn_data(base_path: &Path) -> Result<(), DepsError> {
    get_package("mdn-data", &deps().mdn_data, base_path)?;
    Ok(())
}
