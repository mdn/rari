use rari_templ_func::rari_f;

use crate::error::DocError;

#[rari_f(crate::Templ)]
pub fn echo(s: String) -> Result<String, DocError> {
    Ok(s)
}
