use std::path::Path;

use crate::error::DepsError;
use crate::external_json::get_json;

pub fn update_developer_signals(base_path: &Path) -> Result<(), DepsError> {
    get_json(
        "developer_signals",
        "https://web-platform-dx.github.io/developer-signals/web-features-signals.json",
        base_path,
    )?;
    Ok(())
}
