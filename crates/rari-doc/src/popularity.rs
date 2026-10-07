//! Aggregates page popularity across redirects.

use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexMap;
use rari_types::globals::popularities;

use crate::redirects::REDIRECTS;

/// Inverted redirect map: lowercase target URL to lowercase source URLs.
///
/// Redirect files do not contain chains, so a single hop is sufficient.
static REDIRECTS_REVERSED: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let mut reversed: HashMap<String, Vec<String>> = HashMap::new();
    for (from, to) in REDIRECTS.iter() {
        reversed
            .entry(to.to_lowercase())
            .or_default()
            .push(from.clone());
    }
    reversed
});

/// Returns the page's own page views plus the page views of every URL that
/// redirects to it, or `None` if none of those URLs appear in `popularities.json`.
pub fn popularity_for(url: &str) -> Option<f64> {
    aggregate_popularity(url, &popularities().popularities, &REDIRECTS_REVERSED)
}

fn aggregate_popularity(
    url: &str,
    popularities: &IndexMap<String, f64>,
    redirects_reversed: &HashMap<String, Vec<String>>,
) -> Option<f64> {
    let lower_url = url.to_lowercase();
    let own = popularities.get(&lower_url).copied();
    let extras: f64 = redirects_reversed
        .get(&lower_url)
        .into_iter()
        .flatten()
        .filter_map(|src| popularities.get(src).copied())
        .sum();
    match (own, extras) {
        (None, 0.0) => None,
        (own, extras) => Some(own.unwrap_or(0.0) + extras),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn redirects(entries: &[(&str, &str)]) -> HashMap<String, String> {
        entries
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.to_string()))
            .collect()
    }

    fn reverse(redirects: &HashMap<String, String>) -> HashMap<String, Vec<String>> {
        let mut reversed: HashMap<String, Vec<String>> = HashMap::new();
        for (from, to) in redirects.iter() {
            reversed
                .entry(to.to_lowercase())
                .or_default()
                .push(from.clone());
        }
        reversed
    }

    fn pops(entries: &[(&str, f64)]) -> IndexMap<String, f64> {
        entries
            .iter()
            .map(|(k, v)| (k.to_lowercase(), *v))
            .collect()
    }

    #[test]
    fn popularity_no_redirects_returns_own() {
        let r = reverse(&redirects(&[]));
        let p = pops(&[("/en-US/docs/Web/HTML", 1.0)]);
        assert_eq!(
            aggregate_popularity("/en-us/docs/web/html", &p, &r),
            Some(1.0)
        );
    }

    #[test]
    fn popularity_sums_single_redirect_source() {
        let r = reverse(&redirects(&[(
            "/en-US/docs/Web/Foo/bar",
            "/en-US/docs/Web/Reference/Foo/Bar",
        )]));
        let p = pops(&[
            ("/en-US/docs/Web/Reference/Foo/Bar", 1.0),
            ("/en-US/docs/Web/Foo/bar", 2.0),
        ]);
        assert_eq!(
            aggregate_popularity("/en-us/docs/web/reference/foo/bar", &p, &r),
            Some(3.0)
        );
    }

    #[test]
    fn popularity_sums_multiple_redirect_sources() {
        let r = reverse(&redirects(&[
            ("/en-US/docs/A", "/en-US/docs/Target"),
            ("/en-US/docs/B", "/en-US/docs/Target"),
        ]));
        let p = pops(&[
            ("/en-US/docs/Target", 1.0),
            ("/en-US/docs/A", 2.0),
            ("/en-US/docs/B", 4.0),
        ]);
        assert_eq!(
            aggregate_popularity("/en-us/docs/target", &p, &r),
            Some(7.0)
        );
    }

    #[test]
    fn popularity_only_from_redirect_sources() {
        let r = reverse(&redirects(&[("/en-US/docs/Old", "/en-US/docs/New")]));
        let p = pops(&[("/en-US/docs/Old", 2.5)]);
        assert_eq!(aggregate_popularity("/en-us/docs/new", &p, &r), Some(2.5));
    }

    #[test]
    fn popularity_returns_none_when_absent() {
        let r = reverse(&redirects(&[]));
        let p = pops(&[("/en-US/docs/Other", 1.0)]);
        assert_eq!(aggregate_popularity("/en-us/docs/missing", &p, &r), None);
    }

    #[test]
    fn popularity_for_unrelated_url_with_redirects_returns_own() {
        let r = reverse(&redirects(&[("/en-US/docs/Foo", "/en-US/docs/Bar")]));
        let p = pops(&[("/en-US/docs/Bar", 1.0), ("/en-US/docs/Other", 2.0)]);
        assert_eq!(aggregate_popularity("/en-us/docs/other", &p, &r), Some(2.0));
    }

    #[test]
    fn popularity_normalizes_url_casing_before_lookup() {
        let r = reverse(&redirects(&[(
            "/en-US/docs/Old/Page",
            "/en-US/docs/New/Page",
        )]));
        let p = pops(&[("/en-US/docs/New/Page", 1.0), ("/en-US/docs/Old/Page", 3.0)]);
        assert_eq!(
            aggregate_popularity("/en-US/docs/New/Page", &p, &r),
            Some(4.0)
        );
    }
}
