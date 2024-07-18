pub mod apiref;
pub mod badges;
pub mod banners;
pub mod compat;
pub mod cssinfo;
pub mod csssyntax;
pub mod embedghlivesample;
pub mod embedinteractiveexample;
pub mod glossary;
pub mod inheritance_diagram;
pub mod inline_labels;
pub mod links;
pub mod listsubpages;
pub mod livesample;
pub mod previous_menu_next;
pub mod quick_links_with_subpages;
pub mod specification;
pub mod web_ext_examples;

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
    let name = ident.to_lowercase();
    let is_sidebar = matches!(name.as_str(), "apiref" | "defaultapisidebar");
    let f = match name.as_str() {
        "compat" => compat::compat_any,
        "specifications" => specification::specification_any,
        "glossary" => glossary::glossary_any,
        "csssyntax" => csssyntax::csssyntax_any,
        "embedinteractiveexample" => embedinteractiveexample::embed_interactive_example_any,
        "embedghlivesample" => embedghlivesample::embed_gh_live_sample_any,
        "listsubpages" => listsubpages::list_sub_pages_any,
        "listsubpagesgrouped" => listsubpages::list_sub_pages_grouped_any,
        "embedlivesample" => livesample::live_sample_any,
        "cssinfo" => cssinfo::cssinfo_any,
        "quicklinkswithsubpages" => quick_links_with_subpages::quick_links_with_subpages_any,
        "inheritancediagram" => inheritance_diagram::inheritance_diagram_any,
        "webextexamples" => web_ext_examples::web_ext_examples_any,
        "previousmenunext" => previous_menu_next::previous_menu_next_any,

        // badges
        "experimentalbadge" | "experimental_inline" => badges::experimental_any,
        "nonstandardbadge" | "non-standard_inline" => badges::non_standard_any,
        "deprecated_inline" => badges::deprecated_any,
        "optional_inline" => badges::optional_any,

        //inline labels
        "readonlyinline" => inline_labels::readonly_inline_any,
        "securecontext_inline" => inline_labels::secure_context_inline_any,

        //banners
        "seecompattable" => banners::see_compat_table_any,
        "securecontext_header" => banners::secure_context_header_any,
        "availableinworkers" => banners::available_in_workers_any,
        "deprecated_header" => banners::deprecated_header_any,

        // links
        "csp" => links::csp::csp_any,
        "rfc" => links::rfc::rfc_any,
        "httpheader" => links::http_header::http_header_any,
        "cssxref" => links::cssxref::cssxref_any,
        "jsxref" => links::jsxref::jsxref_any,
        "domxref" => links::domxref::domxref_any,
        "htmlelement" => links::htmlxref::htmlxref_any,
        "svgelement" => links::svgxref::svgxref_any,
        "svgattr" => links::svgattr::svgattr_any,
        "webextapiref" => links::webextapixref::webextapixref_any,

        // sidebars
        "apiref" => apiref::apiref_any,
        "defaultapisidebar" => apiref::default_api_sidebar_any,

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
