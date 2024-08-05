use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::html::sidebar;
use crate::sidebars::{apiref, default_api_sidebar, jsref};

#[rari_f]
pub fn apiref(group: Option<String>) -> Result<String, DocError> {
    apiref::sidebar(env.slug, group.as_deref(), env.locale)?.render(env.locale)
}

#[rari_f]
pub fn default_api_sidebar(group: String) -> Result<String, DocError> {
    default_api_sidebar::sidebar(&group, env.locale)?.render(env.locale)
}

#[rari_f]
pub fn jsref() -> Result<String, DocError> {
    jsref::sidebar(env.slug, env.locale)?.render(env.locale)
}

#[rari_f]
pub fn cssref() -> Result<String, DocError> {
    sidebar::render_sidebar("cssref", env.slug, env.locale)
}

#[rari_f]
pub fn glossarysidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("glossarysidebar", env.slug, env.locale)
}

#[rari_f]
pub fn addonsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("addonsidebar", env.slug, env.locale)
}
