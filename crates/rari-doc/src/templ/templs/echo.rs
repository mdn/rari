use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f]
pub fn echo(s: String) -> Result<String, DocError> {
    Ok(s)
}
