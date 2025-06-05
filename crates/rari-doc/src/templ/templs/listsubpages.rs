use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::helpers::subpages::{self, ListSubPagesContext, SubPagesSorter};

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
            Some(SubPagesSorter::Title),
            &[],
            false,
        )?;
    } else {
        subpages::list_sub_pages_nested_internal(
            &mut out,
            url,
            env.locale,
            Some(depth),
            ListSubPagesContext {
                sorter: Some(SubPagesSorter::Title),
                page_types: &[],
                code: false,
                include_parent: false,
            },
        )?;
    }
    out.push_str(if ordered { "</ol>" } else { "</ul>" });

    Ok(out)
}
