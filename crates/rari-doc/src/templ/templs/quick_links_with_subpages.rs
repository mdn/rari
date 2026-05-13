use rari_templ_func::rari_f;
use rari_types::{AnyArg, Arg};

use super::listsubpages::listsubpages;
use crate::error::DocError;

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
