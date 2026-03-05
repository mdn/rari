//! # Baseline Module
//!
//! The `baseline` module provides functionality for managing and accessing baseline support status
//! for web features. It includes utilities for loading baseline data from files and retrieving
//! support status for specific browser compatibility keys.
use std::sync::LazyLock;

use rari_data::baseline::{Baseline, WebFeatures};
use rari_types::globals::data_dir;
use tracing::error;

static WEB_FEATURES: LazyLock<Option<WebFeatures>> = LazyLock::new(|| {
    let web_features = WebFeatures::from_file(&data_dir().join("web-features/package/data.json"));
    match web_features {
        Ok(web_features) => Some(web_features),
        Err(e) => {
            error!("Failed to load web-features data: {e:?}");
            None
        }
    }
});

/// Retrieves the baseline support status for the given browser compatibility keys.
///
/// When a page lists multiple keys, the **lowest** baseline status among them is used:
/// 1. If any key is missing from web-features, no banner is shown (`None`).
/// 2. If all keys are present but any is `false` (not baseline), the result is `false`.
/// 3. If all are present and none is `false` but any is `low`, the result is `low`.
/// 4. Otherwise all are `high`, and the result is `high`.
///
/// # Arguments
///
/// * `browser_compat` - The browser compatibility keys for the page. All must be present in
///   web-features; if any is missing, this returns `None`.
///
/// # Returns
///
/// * `Option<Baseline>` - The combined baseline (lowest of all keys), or `None` if any key
///   is missing or `WEB_FEATURES` is not initialized.
pub(crate) fn get_baseline<'a>(browser_compat: &[String]) -> Option<Baseline<'a>> {
    let wf = WEB_FEATURES.as_ref();
    let web_features = wf.as_ref()?;
    if browser_compat.is_empty() {
        return None;
    }
    let baselines: Vec<Baseline<'_>> = browser_compat
        .iter()
        .filter_map(|bcd_key| web_features.baseline_by_bcd_key(bcd_key.as_str()))
        .collect();
    if baselines.len() != browser_compat.len() {
        return None;
    }
    baselines
        .into_iter()
        // max to get the "lowest" baseline (worst status)
        .max_by_key(|b| b.support.baseline)
}
