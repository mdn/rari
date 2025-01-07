//! # Baseline Module
//!
//! The `baseline` module provides functionality for managing and accessing baseline support status
//! for web features. It includes utilities for loading baseline data from files and retrieving
//! support status for specific browser compatibility keys.
use std::sync::LazyLock;

use rari_data::baseline::{SupportStatusWithByKey, WebFeatures};
use rari_types::globals::data_dir;
use tracing::warn;

static WEB_FEATURES: LazyLock<Option<WebFeatures>> = LazyLock::new(|| {
    let web_features =
        WebFeatures::from_file(&data_dir().join("baseline").join("data.extended.json"));
    match web_features {
        Ok(web_features) => Some(web_features),
        Err(e) => {
            warn!("{e:?}");
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
pub(crate) fn get_baseline(
    browser_compat: &[String],
) -> Option<(&'static SupportStatusWithByKey, bool)> {
    if let Some(ref web_features) = *WEB_FEATURES {
        return match &browser_compat {
            &[bcd_key] => web_features.feature_status(bcd_key.as_str()),
            _ => None,
        };
    }
    None
}
