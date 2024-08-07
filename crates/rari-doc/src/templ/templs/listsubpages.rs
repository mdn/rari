use std::str::FromStr;

use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::helpers::subpages::{self, SubPagesSorter};

/// List sub pages
#[rari_f]
pub fn list_sub_pages(
    url: Option<String>,
    depth: Option<AnyArg>,
    reverse: Option<AnyArg>,
    ordered: Option<AnyArg>,
) -> Result<String, DocError> {
    let depth = depth.map(|d| d.as_int() as usize).unwrap_or(1);
    let url = url.as_deref().filter(|s| !s.is_empty()).unwrap_or(env.url);
    let ordered = ordered.as_ref().map(AnyArg::as_bool).unwrap_or_default();
    let mut out = String::new();
    out.push_str(if ordered { "<ol>" } else { "<ul>" });
    if reverse.map(|r| r.as_int() != 0).unwrap_or_default() {
        // Yes the old marco checks for == 0 not === 0.
        if depth > 1 {
            return Err(DocError::InvalidTempl(
                "listsubpages with reverse set and depth != 1".to_string(),
            ));
        }
        subpages::list_sub_pages_reverse_internal(
            &mut out,
            url,
            env.locale,
            Some(SubPagesSorter::SlugNatural),
            &[],
        )?;
    } else {
        subpages::list_sub_pages_internal(
            &mut out,
            url,
            env.locale,
            Some(depth),
            Some(SubPagesSorter::SlugNatural),
            &[],
        )?;
    }
    out.push_str(if ordered { "</ol>" } else { "</ul>" });

    Ok(out)
}

#[rari_f]
pub fn list_sub_pages_grouped(
    url: Option<String>,
    title: Option<String>,
    page_types: Option<String>,
) -> Result<String, DocError> {
    let url = url.as_deref().unwrap_or(env.url);
    let title = title.as_deref().unwrap_or(env.title);
    let mut out = String::new();
    out.push_str("<details><summary>");
    out.push_str(&html_escape::encode_safe(title));
    out.push_str("</summary><ol>");
    subpages::list_sub_pages_grouped_internal(
        &mut out,
        url,
        env.locale,
        None,
        page_types
            .map(|pt| {
                pt.split(',')
                    .filter_map(|pt| PageType::from_str(pt.trim()).ok())
                    .collect::<Vec<_>>()
            })
            .as_deref()
            .unwrap_or_default(),
    )?;
    out.push_str("</ol></details>");
    Ok(out)
}
