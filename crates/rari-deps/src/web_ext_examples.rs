use std::path::Path;

use crate::error::DepsError;
use crate::external_json::get_json;

pub fn update_web_ext_examples(base_path: &Path) -> Result<(), DepsError> {
    get_json(
        "web_ext_examples",
        "https://raw.githubusercontent.com/mdn/webextensions-examples/main/examples.json",
        base_path,
    )?;
    Ok(())
}
