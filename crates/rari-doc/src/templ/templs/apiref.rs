use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::sidebars::{apiref, default_api_sidebar};

#[rari_f]
pub fn apiref(group: Option<String>) -> Result<String, DocError> {
    apiref::sidebar(env.slug, group.as_deref(), env.locale)?.render(env.locale)
}

#[rari_f]
pub fn default_api_sidebar(group: String) -> Result<String, DocError> {
    default_api_sidebar::sidebar(&group, env.locale)?.render(env.locale)
}
