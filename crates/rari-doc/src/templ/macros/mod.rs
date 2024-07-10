pub mod apiref;
pub mod badges;
pub mod compat;
pub mod cssinfo;
pub mod csssyntax;
pub mod cssxref;
pub mod domxref;
pub mod embedinteractiveexample;
pub mod glossary;
pub mod htmlxref;
pub mod jsxref;
pub mod links;
pub mod listsubpages;
pub mod livesample;
pub mod quick_links_with_subpages;
pub mod specification;
pub mod svgxref;

use rari_types::globals::deny_warnings;
use rari_types::{Arg, RariEnv};
use tracing::error;

use crate::error::DocError;
use crate::utils::TEMPL_RECORDER;

pub fn invoke(
    env: &RariEnv,
    ident: &str,
    args: Vec<Option<Arg>>,
) -> Result<(String, bool), DocError> {
    let is_sidebar = matches!(ident.to_lowercase().as_str(), "apiref");
    let f = match ident.to_lowercase().as_str() {
        "compat" => compat::compat_any,
        "specifications" => specification::specification_any,
        "glossary" => glossary::glossary_any,
        "csssyntax" => csssyntax::csssyntax_any,
        "embedinteractiveexample" => embedinteractiveexample::embed_interactive_example_any,
        "listsubpages" => listsubpages::list_sub_pages_any,
        "listsubpagesgrouped" => listsubpages::list_sub_pages_grouped_any,
        "embedlivesample" => livesample::live_sample_any,
        "cssinfo" => cssinfo::cssinfo_any,
        "quicklinkswithsubpages" => quick_links_with_subpages::quick_links_with_subpages_any,

        // badges
        "experimentalbadge" | "experimental_inline" => badges::experimental_any,
        "nonstandardbadge" | "non-standard_inline" => badges::non_standard_any,
        "deprecated_inline" => badges::deprecated_any,
        "optional_inline" => badges::optional_any,

        // links
        "link" => links::link_any,
        "doclink" => links::doc_link_any,
        "csp" => links::csp_any,
        "rfc" => links::rfc_any,
        "httpheader" => links::http_header_any,

        // xrefs
        "cssxref" => cssxref::cssxref_any,
        "jsxref" => jsxref::jsxref_any,
        "domxref" => domxref::domxref_any,
        "htmlelement" => htmlxref::htmlxref_any,
        "svgelement" => svgxref::svgxref_any,

        // sidebars
        "apiref" => apiref::apiref_any,

        // ignore
        "cssref" | "glossarysidebar" | "jsref" => return Ok(Default::default()),

        // unknown
        _ if deny_warnings() => return Err(DocError::UnknownMacro(ident.to_string())),
        _ => {
            TEMPL_RECORDER.with(|tx| {
                if let Some(tx) = tx {
                    if let Err(e) = tx.send(ident.to_string()) {
                        error!("templ recorder: {e}");
                    }
                }
            });
            return Ok((format!("<s>unsupported templ: {ident}</s>"), is_sidebar));
        } //
    };
    f(env, args).map(|s| (s, is_sidebar))
}
