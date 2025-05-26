use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::html::sidebar;
use crate::sidebars::{apiref, default_api_sidebar, jsref};

#[rari_f(crate::Templ)]
pub fn apiref(group: Option<String>) -> Result<String, DocError> {
    apiref::sidebar(env.slug, group.as_deref(), env.locale)?.render("apiref", env.locale)
}

#[rari_f(crate::Templ)]
pub fn default_api_sidebar(group: String) -> Result<String, DocError> {
    default_api_sidebar::sidebar(&group, env.locale)?.render("default_api", env.locale)
}

#[rari_f(crate::Templ)]
pub fn jsref() -> Result<String, DocError> {
    jsref::sidebar(env.slug, env.locale)?.render("jsref", env.locale)
}

#[rari_f(crate::Templ)]
pub fn cssref() -> Result<String, DocError> {
    sidebar::render_sidebar("cssref", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn glossarysidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("glossarysidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn addonsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("addonsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn learnsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("learnsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn svgref() -> Result<String, DocError> {
    sidebar::render_sidebar("svgref", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn httpsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("httpsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn jssidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("jssidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn htmlsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("htmlsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn accessibilitysidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("accessibilitysidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn firefoxsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("firefoxsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn webassemblysidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("webassemblysidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn xsltsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("xsltsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn mdnsidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("mdnsidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn gamessidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("gamessidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn mathmlref() -> Result<String, DocError> {
    sidebar::render_sidebar("mathmlref", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn pwasidebar() -> Result<String, DocError> {
    sidebar::render_sidebar("pwasidebar", env.slug, env.locale)
}

#[rari_f(crate::Templ)]
pub fn addonsidebarmain() -> Result<String, DocError> {
    sidebar::render_sidebar("addonsidebarmain", env.slug, env.locale)
}
