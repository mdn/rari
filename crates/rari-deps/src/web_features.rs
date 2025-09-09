use std::path::Path;

use rari_types::globals::deps;

use crate::error::DepsError;
use crate::github_release::get_artifact;

pub fn update_web_features(base_path: &Path) -> Result<(), DepsError> {
    //get_package("web-features", None, base_path)?;

    get_artifact(
        "web-platform-dx/web-features",
        "data.extended.json",
        "baseline",
        &deps().web_features,
        base_path,
    )?;
    Ok(())
}
