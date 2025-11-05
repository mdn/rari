use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;

use crate::error::DocError;
use crate::helpers::subpages::{get_sub_pages, SubPagesSorter};
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
            && let Page::Doc(doc) = page {
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

fn compat_internal(browser_compat: &[impl AsRef<str>]) -> String {
    let multiple = browser_compat.len() > 1;
    browser_compat
        .iter()
        .map(|query| {
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
    use crate::templ::render::{decode_ref, render, Rendered};

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
