use rari_templ_func::rari_f;
use rari_types::{AnyArg, Arg};

use super::listsubpages::listsubpages;
use crate::error::DocError;
use crate::templ::legacy::normalize_and_check_url_arg;

/// List sub pages
#[rari_f(register = "crate::Templ")]
pub fn quicklinkswithsubpages(url: Option<String>) -> Result<String, DocError> {
    let url = match url {
        Some(s) => match normalize_and_check_url_arg(&s, env.locale) {
            Some(url) => Some(url),
            None => return Ok(String::new()),
        },
        None => None,
    };

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
