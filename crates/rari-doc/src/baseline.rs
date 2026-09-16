//! # Baseline Module
//!
//! The `baseline` module provides functionality for managing and accessing baseline support status
//! for web features. It includes utilities for loading baseline data from files and retrieving
//! support status for specific browser compatibility keys.
use std::sync::LazyLock;

use rari_data::baseline::{Baseline, BaselineStatus, WebFeatures};
use rari_types::fm_types::FeatureStatus;
use rari_types::globals::data_dir;
use tracing::error;

use crate::pages::page::Page;

static WEB_FEATURES: LazyLock<Option<WebFeatures>> = LazyLock::new(|| {
    let web_features = WebFeatures::from_file(&data_dir().join("web-features/package/data.json"));
    match web_features {
        Ok(mut web_features) => {
            if let Err(e) =
                web_features.load_developer_signals(&data_dir().join("developer_signals/data.json"))
            {
                error!("Failed to load developer-signals data: {e:?}");
            }
            Some(web_features)
        }
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

pub(crate) fn is_mocked_banner_section(slug: &str) -> bool {
    // only mock banners under Web and WebAssembly, as those are the sections we have Baseline banners
    // don't mock under Web/Accessibility, as we don't have Baseline banners there
    ["Web/", "WebAssembly/"]
        .iter()
        .any(|prefix| slug.starts_with(prefix))
        && !slug.starts_with("Web/Accessibility/")
}

pub(crate) fn get_mocked_baseline_status(
    fm_status: &[FeatureStatus],
    slug: &str,
) -> Option<BaselineStatus> {
    if !fm_status.contains(&FeatureStatus::Deprecated) {
        return None;
    }
    if !is_mocked_banner_section(slug) {
        return None;
    }
    Some(BaselineStatus::Discouraged)
}

pub(crate) fn get_baseline_status(page: &Page) -> Option<BaselineStatus> {
    let doc = match page {
        Page::Doc(doc) => doc,
        _ => return None,
    };
    get_baseline(&doc.meta.browser_compat)
        .map(|baseline| baseline.status())
        .or_else(|| get_mocked_baseline_status(&doc.meta.status, &doc.meta.slug))
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
    use rari_types::locale::Locale;

    use super::*;
    use crate::pages::types::spa::SPA;

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
    fn non_doc_pages_have_no_status() {
        let page = SPA::from_slug("blog", Locale::EnUs).unwrap();
        assert!(get_baseline_status(&page).is_none());
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

    fn mocked(slug: &str) -> Option<BaselineStatus> {
        get_mocked_baseline_status(&[FeatureStatus::Deprecated], slug)
    }

    #[test]
    fn a_deprecated_page_in_a_baseline_section_is_discouraged() {
        for slug in [
            "Web/API/AudioProcessingEvent",
            "WebAssembly/JavaScript_interface/Memory",
        ] {
            assert_eq!(mocked(slug), Some(BaselineStatus::Discouraged), "{slug}");
        }
    }

    #[test]
    fn a_deprecated_page_outside_the_baseline_sections_is_not_mocked() {
        for slug in [
            "Mozilla/Add-ons/WebExtensions/API/tabs",
            "Games/Techniques/3D_on_the_web",
            "Learn_web_development/Core/Scripting",
        ] {
            assert_eq!(mocked(slug), None, "{slug}");
        }
    }

    #[test]
    fn the_excluded_section_is_not_mocked() {
        assert_eq!(
            mocked("Web/Accessibility/ARIA/Reference/Attributes/aria-dropeffect"),
            None
        );
    }

    #[test]
    fn a_page_without_deprecated_status_is_not_mocked() {
        let statuses: [&[FeatureStatus]; 2] = [&[], &[FeatureStatus::Experimental]];
        for status in statuses {
            assert_eq!(
                get_mocked_baseline_status(status, "Web/API/Thing"),
                None,
                "{status:?}"
            );
        }
    }
}
