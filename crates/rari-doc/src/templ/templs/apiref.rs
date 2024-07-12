use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::sidebars::apiref;

#[rari_f]
pub fn apiref(group: Option<String>) -> Result<String, DocError> {
    apiref::sidebar(env.slug, group.as_deref(), env.locale)?.render(env.locale)
}
