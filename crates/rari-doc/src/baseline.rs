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

/// Retrieves the baseline support status for a given browser compatibility key.
///
/// This function looks up the baseline support status for the provided browser compatibility key
/// in the `WEB_FEATURES` static variable. If it contains the specified key, it returns the corresponding
/// `SupportStatusWithByKey`. If the key is not found, it returns `None`.
///
/// # Arguments
///
/// * `browser_compat` - A slice of strings that holds the browser compatibility keys to be looked up. This function
///   only deals with single keys, so the slice should contain only one element.
///
/// # Returns
///
/// * `Option<&'static SupportStatusWithByKey>` - Returns `Some(&SupportStatusWithByKey)` if the key is found,
///   or `None` if the key is not found or `WEB_FEATURES` is not initialized.
pub(crate) fn get_baseline<'a>(browser_compat: &[String]) -> Option<Baseline<'a>> {
    if let Some(ref web_features) = *WEB_FEATURES {
        return match &browser_compat {
            &[bcd_key] => web_features.baseline_by_bcd_key(bcd_key.as_str()),
            _ => None,
        };
    }
    None
}
