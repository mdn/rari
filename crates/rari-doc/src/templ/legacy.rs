use rari_types::locale::Locale;

use crate::issues::get_issue_counter;
use crate::pages::page::Page;

/// Normalize a slug or URL argument to `/{locale}/docs/...`. If `input`
/// already starts with the locale+docs prefix it is returned as-is.
fn normalize_url(input: &str, locale: Locale) -> String {
    if input.is_empty() {
        return String::new();
    }
    let prefix = format!("/{}/docs", locale.as_url_str());
    if input.starts_with(&prefix) {
        input.to_string()
    } else {
        format!("{prefix}/{}", input.trim_start_matches('/'))
    }
}

/// Normalize a slug or URL argument to `/{locale}/docs/...` and verify the
/// page exists. Returns `None` and emits a `templ-invalid-arg` issue if the
/// resolved URL does not point to an existing page.
pub fn normalize_and_check_url_arg(input: &str, locale: Locale) -> Option<String> {
    let url = normalize_url(input, locale);
    if Page::from_url_with_fallback(&url).is_err() {
        let ic = get_issue_counter();
        tracing::warn!(source = "templ-invalid-arg", ic = ic, arg = url);
        return None;
    }
    Some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_slug_gets_locale_and_docs_prefix() {
        assert_eq!(
            normalize_url("Web/Performance", Locale::EnUs),
            "/en-US/docs/Web/Performance"
        );
        assert_eq!(
            normalize_url("Web/Performance", Locale::Fr),
            "/fr/docs/Web/Performance"
        );
    }

    #[test]
    fn leading_slash_is_stripped_before_prefixing() {
        assert_eq!(
            normalize_url("/Web/Performance", Locale::Fr),
            "/fr/docs/Web/Performance"
        );
    }

    #[test]
    fn already_prefixed_url_is_returned_as_is() {
        assert_eq!(
            normalize_url("/fr/docs/Web/Performance", Locale::Fr),
            "/fr/docs/Web/Performance"
        );
        assert_eq!(
            normalize_url("/fr/docs/Web/Performance/", Locale::Fr),
            "/fr/docs/Web/Performance/"
        );
    }

    #[test]
    fn prefix_check_is_locale_specific() {
        // A `/fr/docs/...` URL processed under en-US is treated as a slug
        // and gets re-prefixed.
        assert_eq!(
            normalize_url("/fr/docs/Web", Locale::EnUs),
            "/en-US/docs/fr/docs/Web"
        );
    }

    #[test]
    fn empty_input_yields_empty_string() {
        assert_eq!(normalize_url("", Locale::EnUs), "");
    }
}
