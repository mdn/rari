use std::collections::HashSet;
use std::sync::LazyLock;

use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::globals::data_dir;

use crate::error::DocError;
use crate::helpers::subpages::{SubPagesSorter, get_sub_pages};
use crate::issues::get_issue_counter;
use crate::pages::page::{Page, PageLike};

#[rari_f(register = "crate::Templ")]
pub fn compat() -> Result<String, DocError> {
    Ok(compat_internal(env.browser_compat))
}

#[rari_f(register = "crate::Templ")]
pub fn webextallcompattables() -> Result<String, DocError> {
    let mut out = String::new();
    let sub_pages = get_sub_pages(
        "/en-US/docs/Mozilla/Add-ons/WebExtensions/API",
        Some(1),
        SubPagesSorter::default(),
    )?;
    for page in sub_pages.iter().filter_map(|page| {
        if page.page_type() == PageType::WebextensionApi
            && let Page::Doc(doc) = page
        {
            return Some(doc);
        }
        None
    }) {
        for feature_name in &page.meta.browser_compat {
            out.extend([
                "<h2>",
                feature_name
                    .as_str()
                    .strip_prefix("webextensions.api.")
                    .unwrap_or(feature_name.as_str()),
                "</h2>",
            ]);
            out.push_str(&compat_internal(&[feature_name]));
        }
    }
    Ok(out)
}

static BCD_KEYS: LazyLock<Option<HashSet<String>>> = LazyLock::new(|| {
    let path = data_dir().join("@mdn/browser-compat-data/bcd_keys.json");
    let keys = std::fs::read_to_string(path)
        .map_err(DocError::from)
        .and_then(|data| Ok(serde_json::from_str(&data)?));
    match keys {
        Ok(keys) => Some(keys),
        Err(error) => {
            tracing::error!("Failed to load BCD keys: {error}");
            None
        }
    }
});

fn compat_internal(browser_compat: &[impl AsRef<str>]) -> String {
    if browser_compat.is_empty() {
        return String::new();
    }
    compat_with_keys(browser_compat, BCD_KEYS.as_ref())
}

fn compat_with_keys(browser_compat: &[impl AsRef<str>], keys: Option<&HashSet<String>>) -> String {
    let multiple = browser_compat.len() > 1;
    browser_compat
        .iter()
        .map(|query| {
            if let Some(keys) = keys
                && !keys.contains(query.as_ref())
            {
                tracing::warn!(
                    source = "templ-bcd-missing",
                    ic = get_issue_counter(),
                    query = query.as_ref(),
                    "Unknown browser-compat entry: {}",
                    query.as_ref()
                );
            }
            format!(
                r#"<div class="bc-data" data-query="{}" data-depth="1" data-multiple="{multiple}">
If you're able to see this, something went wrong on this page.
</div>"#,
                query.as_ref()
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod test {
    use rari_types::RariEnv;

    use crate::error::DocError;
    use crate::templ::render::{Rendered, decode_ref, render};

    #[test]
    fn test_compat_bcd_keys() {
        use tracing_subscriber::layer::SubscriberExt;

        use super::compat_with_keys;
        use crate::issues::InMemoryLayer;

        let keys = [
            "css",
            "css.types",
            "css.types.color",
            "css.types.color.color-mix",
            "css.types.color.color",
            "css.types.color.color.display-p3",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let valid = "css.types.color.color-mix";
        let invalid = "css.types.color.color.display-p3-linear";
        let cases = vec![
            ("empty", vec![], vec![]),
            ("valid", vec![valid], vec![]),
            ("group", vec!["css.types.color"], vec![]),
            ("missing", vec![invalid], vec![invalid]),
            ("mixed", vec![valid, invalid], vec![invalid]),
            (
                "multiple missing",
                vec![invalid, "css.typo"],
                vec![invalid, "css.typo"],
            ),
            (
                "metadata",
                vec!["browsers.firefox"],
                vec!["browsers.firefox"],
            ),
            (
                "compat metadata",
                vec!["css.types.color.color-mix.__compat"],
                vec!["css.types.color.color-mix.__compat"],
            ),
            ("empty key", vec![""], vec![""]),
        ];
        for (name, queries, expected) in cases {
            let layer = InMemoryLayer::default();
            let subscriber = tracing_subscriber::registry().with(layer.clone());
            let _guard = tracing::subscriber::set_default(subscriber);
            let out = compat_with_keys(&queries, Some(&keys));
            assert_eq!(
                out.matches("class=\"bc-data\"").count(),
                queries.len(),
                "{name}"
            );
            let events = layer.get_events();
            let issues = events
                .get("")
                .map(|issues| issues.clone())
                .unwrap_or_default();
            let actual: Vec<_> = issues
                .iter()
                .map(|issue| {
                    assert!(
                        issue
                            .fields
                            .contains(&("source", "templ-bcd-missing".into())),
                        "{name}"
                    );
                    issue
                        .fields
                        .iter()
                        .find(|(key, _)| *key == "query")
                        .unwrap()
                        .1
                        .as_str()
                })
                .collect();
            assert_eq!(actual, expected, "{name}");
        }
    }

    #[test]
    fn test_compat_none() -> Result<(), DocError> {
        let env = RariEnv {
            ..Default::default()
        };
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{ compat }}"#, 0)?;
        let out = decode_ref(&content, &templs, None)?;
        assert_eq!(out, r#""#);
        Ok(())
    }

    #[test]
    fn test_compat() -> Result<(), DocError> {
        let env = RariEnv {
            browser_compat: &["javascript.builtins.Array.concat".into()],
            ..Default::default()
        };
        let exp = r#"<div class="bc-data" data-query="javascript.builtins.Array.concat" data-depth="1" data-multiple="false">
If you're able to see this, something went wrong on this page.
</div>"#;
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{ compat }}"#, 0)?;
        let out = decode_ref(&content, &templs, None)?;
        assert_eq!(out, exp);
        Ok(())
    }

    #[test]
    fn test_compat_multiple() -> Result<(), DocError> {
        let env = RariEnv {
            browser_compat: &[
                "javascript.builtins.Array.concat".into(),
                "javascript.builtins.Array.filter".into(),
            ],
            ..Default::default()
        };
        let exp = r#"<div class="bc-data" data-query="javascript.builtins.Array.concat" data-depth="1" data-multiple="true">
If you're able to see this, something went wrong on this page.
</div>
<div class="bc-data" data-query="javascript.builtins.Array.filter" data-depth="1" data-multiple="true">
If you're able to see this, something went wrong on this page.
</div>"#;
        let Rendered {
            content, templs, ..
        } = render(&env, r#"{{ compat }}"#, 0)?;
        let out = decode_ref(&content, &templs, None)?;
        assert_eq!(out, exp);
        Ok(())
    }
}
