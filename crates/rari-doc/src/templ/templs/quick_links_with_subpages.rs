use rari_templ_func::rari_f;
use rari_types::{AnyArg, Arg};

use super::listsubpages::list_sub_pages;
use crate::error::DocError;
use crate::templ::legacy::fix_broken_legacy_url;

/// List sub pages
#[rari_f(register = "crate::Templ")]
pub fn quick_links_with_subpages(url: Option<String>) -> Result<String, DocError> {
    let url = url.map(|s| fix_broken_legacy_url(&s, env.locale).to_string());
    list_sub_pages(
        env,
        url,
        Some(AnyArg { value: Arg::Int(2) }),
        None,
        Some(AnyArg {
            value: Arg::Bool(true),
        }),
    )
}
