use std::borrow::Cow;
use std::str::FromStr;

use rari_types::locale::Locale;
use tracing::warn;

pub fn fix_broken_legacy_url(url: &str, locale: Locale) -> Cow<'_, str> {
    let first_non_slash = url.find(|c| c != '/').unwrap_or(url.len());
    let trimmed = &url[first_non_slash..];
    let maybe_locale = &trimmed[..trimmed.find('/').unwrap_or(trimmed.len())];
    let parsed_locale = Locale::from_str(maybe_locale);
    let has_locale = parsed_locale.is_ok();
    let (has_docs, maybe_without_docs) = if has_locale {
        let without_locale = trimmed.strip_prefix(maybe_locale).unwrap_or(trimmed);
        (
            without_locale.starts_with("/docs/"),
            without_locale.strip_prefix('/').unwrap_or(without_locale),
        )
    } else {
        (trimmed.starts_with("docs/"), trimmed)
    };

    let fixed_url = match (first_non_slash > 0, has_locale, has_docs) {
        (true, true, true) => Cow::Borrowed(&url[first_non_slash - 1..]),
        (true, true, false) => Cow::Owned(["", maybe_locale, "docs", maybe_without_docs].join("/")),
        (false, true, true) => Cow::Owned(["", url].join("/")),
        (false, true, false) => {
            Cow::Owned(["", maybe_locale, "docs", maybe_without_docs].join("/"))
        }
        (_, false, true) => Cow::Owned(["", locale.as_url_str(), trimmed].join("/")),
        (_, false, false) => {
            Cow::Owned(["", locale.as_url_str(), "docs", maybe_without_docs].join("/"))
        }
    };
    if fixed_url != url {
        warn!("fixed legacy url: {url} -> {fixed_url}");
    }
    fixed_url
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fix_broken_legacy_url() {
        assert_eq!(
            fix_broken_legacy_url("/en-US/docs/Web", Locale::EnUs),
            "/en-US/docs/Web"
        );
        assert_eq!(
            fix_broken_legacy_url("/docs/Web", Locale::EnUs),
            "/en-US/docs/Web"
        );
        assert_eq!(
            fix_broken_legacy_url("/en-US/Web", Locale::EnUs),
            "/en-US/docs/Web"
        );
        assert_eq!(
            fix_broken_legacy_url("//Web", Locale::EnUs),
            "/en-US/docs/Web"
        );
        assert_eq!(
            fix_broken_legacy_url("//en-US/Web", Locale::EnUs),
            "/en-US/docs/Web"
        );
        assert_eq!(
            fix_broken_legacy_url("/Web", Locale::EnUs),
            "/en-US/docs/Web"
        );
    }
}

/*
function repairURL(url) {
  // Returns a lowercase URI with common irregularities repaired.
  url = url.trim().toLowerCase();
  if (!url.startsWith("/")) {
    // Ensure the URI starts with a "/".
    url = `/${url}`;
  }
  // Remove redundant forward slashes, like "//".
  url = url.replace(/\/{2,}/g, "/");
  // Ensure the URI starts with a valid locale.
  const maybeLocale = url.split("/")[1];
  if (!isValidLocale(maybeLocale)) {
    if (maybeLocale === "en") {
      // Converts URI's like "/en/..." to "/en-us/...".
      url = url.replace(`/${maybeLocale}`, "/en-us");
    } else {
      // Converts URI's like "/web/..." to "/en-us/web/...", or
      // URI's like "/docs/..." to "/en-us/docs/...".
      url = `/en-us${url}`;
    }
  }
  // Ensure the locale is followed by "/docs".
  const [locale, maybeDocs] = url.split("/").slice(1, 3);
  if (maybeDocs !== "docs") {
    // Converts URI's like "/en-us/web/..." to "/en-us/docs/web/...".
    url = url.replace(`/${locale}`, `/${locale}/docs`);
  }
  return url;
}
*/
