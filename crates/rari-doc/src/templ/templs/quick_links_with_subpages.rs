use rari_templ_func::rari_f;
use rari_types::{AnyArg, Arg};

use super::listsubpages::listsubpages;
use crate::error::DocError;
use crate::issues::get_issue_counter;
use crate::pages::page::Page;

/// List sub pages
#[rari_f(register = "crate::Templ")]
pub fn quicklinkswithsubpages(url: Option<String>) -> Result<String, DocError> {
    let prefix = format!("/{}/docs", env.locale.as_url_str());
    let url = url.map(|s| {
        if s.starts_with(&prefix) {
            s
        } else {
            format!("{prefix}/{}", s.trim_start_matches('/'))
        }
    });

    if let Some(url) = url.as_deref()
        && Page::from_url_with_fallback(url).is_err()
    {
        let ic = get_issue_counter();
        tracing::warn!(source = "templ-invalid-arg", ic = ic, arg = url);
        return Ok(String::new());
    }

    listsubpages(
        env,
        url,
        Some(AnyArg { value: Arg::Int(2) }),
        None,
        Some(AnyArg {
            value: Arg::Bool(true),
        }),
    )
}
