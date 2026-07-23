use std::{collections::HashMap, sync::LazyLock};

// Hand-curated because web-features gives feature IDs with no MDN URL. Only needs
// features that appear as an alternative of a discouraged feature that MDN has pages for.
static FEATURE_MDN_URLS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("feature_urls.json"))
        .expect("feature_urls.json is not valid JSON")
});

pub fn get_mdn_url(feature_key: &str) -> Option<&'static str> {
    FEATURE_MDN_URLS.get(feature_key).map(String::as_str)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn known_key_resolves() {
        assert_eq!(get_mdn_url("svg"), Some("/docs/Web/SVG"));
        assert_eq!(
            get_mdn_url("color"),
            Some("/docs/Web/CSS/Reference/Properties/color")
        );
    }

    #[test]
    fn unknown_key_is_none() {
        assert_eq!(get_mdn_url("does-not-exist"), None);
    }
}
