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

/// Looks up baseline support status for the given browser compatibility keys in `WEB_FEATURES`.
///
/// Returns the baseline for the keys' shared web feature, or `None` if the keys belong to
/// different features or none, or `WEB_FEATURES` is not initialized.
pub(crate) fn get_baseline<'a>(browser_compat: &[String]) -> Option<Baseline<'a>> {
    if let Some(ref web_features) = *WEB_FEATURES {
        return get_baseline_from(browser_compat, web_features);
    }
    None
}

fn get_baseline_from<'a>(
    browser_compat: &[String],
    web_features: &'a WebFeatures,
) -> Option<Baseline<'a>> {
    let first = web_features.baseline_by_bcd_key(browser_compat.first()?.as_str())?;
    browser_compat[1..]
        .iter()
        .all(|key| {
            web_features
                .baseline_by_bcd_key(key.as_str())
                .is_some_and(|b| b.feature.id == first.feature.id)
        })
        .then_some(first)
}

#[cfg(test)]
mod tests {
    use rari_data::baseline::BaselineHighLow;

    use super::*;

    static TEST_WEB_FEATURES: LazyLock<WebFeatures> = LazyLock::new(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/web-features.json");
        WebFeatures::from_file(&path).unwrap()
    });

    fn get(keys: &[&str]) -> Option<Baseline<'static>> {
        let keys: Vec<String> = keys.iter().map(|s| s.to_string()).collect();
        get_baseline_from(&keys, &TEST_WEB_FEATURES)
    }

    #[test]
    fn empty() {
        assert!(get(&[]).is_none());
    }

    #[test]
    fn missing() {
        assert!(get(&["api.NonExistent"]).is_none());
    }

    #[test]
    fn single_high() {
        let b = get(&["api.high"]).unwrap();
        assert_eq!(b.support.baseline, BaselineHighLow::High);
    }

    #[test]
    fn single_low() {
        let b = get(&["api.low"]).unwrap();
        assert_eq!(b.support.baseline, BaselineHighLow::Low);
    }

    #[test]
    fn single_limited() {
        let b = get(&["api.limited"]).unwrap();
        assert_eq!(b.support.baseline, BaselineHighLow::False);
    }

    #[test]
    fn multiple_one_missing() {
        assert!(get(&["api.high", "api.NonExistent"]).is_none());
    }

    #[test]
    fn multiple_different_features() {
        assert!(get(&["api.high", "api.low"]).is_none());
    }

    #[test]
    fn multiple_same_feature() {
        let b = get(&["api.high", "api.high-adjacent"]).unwrap();
        assert_eq!(b.support.baseline, BaselineHighLow::High);
    }
}
