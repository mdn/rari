use std::path::Path;

use crate::error::DepsError;
use crate::github_release::get_artifact;

pub fn update_web_features(base_path: &Path) -> Result<(), DepsError> {
    //get_package("web-features", None, base_path)?;
    get_artifact(
        "https://github.com/web-platform-dx/web-features",
        "data.extended.json",
        "baseline",
        None,
        base_path,
    )?;
    Ok(())
}
