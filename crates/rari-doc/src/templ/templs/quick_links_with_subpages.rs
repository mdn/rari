use rari_templ_func::rari_f;
use rari_types::{AnyArg, Arg};

use super::listsubpages::listsubpages;
use crate::error::DocError;

/// List sub pages
#[rari_f(register = "crate::Templ")]
pub fn quicklinkswithsubpages(url: Option<String>) -> Result<String, DocError> {
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
