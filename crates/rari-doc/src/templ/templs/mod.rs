pub mod badges;
pub mod banners;
pub mod compat;
pub mod cssinfo;
pub mod csssyntax;
pub mod embeds;
pub mod firefox_for_developers;
pub mod glossary;
pub mod glossarydisambiguation;
pub mod inheritance_diagram;
pub mod inline_labels;
pub mod js_property_attributes;
pub mod links;
pub mod listsubpages;
pub mod previous_menu_next;
pub mod quick_links_with_subpages;
pub mod sidebars;
pub mod specification;
pub mod svginfo;
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
    let name = ident.to_ascii_lowercase();

    // TODO: improve sidebar handling
    let is_sidebar = matches!(
        name.as_str(),
        "apiref"
            | "defaultapisidebar"
            | "jsref"
            | "cssref"
            | "glossarysidebar"
            | "quicklinkswithsubpages"
            | "addonsidebar"
    );
    let f = match name.as_str() {
        "compat" => compat::compat_any,
        "specifications" => specification::specification_any,
        "glossary" => glossary::glossary_any,
        "csssyntax" => csssyntax::csssyntax_any,
        "listsubpages" => listsubpages::list_sub_pages_any,
        "listsubpagesgrouped" => listsubpages::list_sub_pages_grouped_any,
        "cssinfo" => cssinfo::cssinfo_any,
        "inheritancediagram" => inheritance_diagram::inheritance_diagram_any,
        "webextexamples" => web_ext_examples::web_ext_examples_any,
        "firefox_for_developers" => firefox_for_developers::firefox_for_developers_any,
        "js_property_attributes" => js_property_attributes::js_property_attributes_any,
        "svginfo" => svginfo::svginfo_any,

        // hacky
        "glossarydisambiguation" => glossarydisambiguation::glossarydisambiguation_any,

        // prev menu next
        "previousmenunext" => previous_menu_next::previous_next_menu_any,
        "previousnext" => previous_menu_next::previous_next_any,
        "previousmenu" => previous_menu_next::previous_menu_any,
        "previous" => previous_menu_next::previous_any,
        "nextmenu" => previous_menu_next::next_menu_any,
        "next" => previous_menu_next::next_any,

        // embeds
        "embedinteractiveexample" => embeds::embedinteractiveexample::embed_interactive_example_any,
        "embedghlivesample" => embeds::embedghlivesample::embed_gh_live_sample_any,
        "embedlivesample" => embeds::livesample::live_sample_any,
        "embedyoutube" => embeds::embedyoutube::embed_youtube_any,
        "jsfiddleembed" => embeds::jsfiddleembed::embded_jsfiddle_any,

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
        "non-standard_header" => banners::non_standard_header_any,

        // links
        "csp" => links::csp::csp_any,
        "rfc" => links::rfc::rfc_any,
        "cssxref" => links::cssxref::cssxref_any,
        "jsxref" => links::jsxref::jsxref_any,
        "domxref" => links::domxref::domxref_any,
        "htmlelement" => links::htmlxref::htmlxref_any,
        "svgelement" => links::svgxref::svgxref_any,
        "svgattr" => links::svgattr::svgattr_any,
        "webextapiref" => links::webextapixref::webextapixref_any,
        "httpstatus" => links::http::http_status_any,
        "httpheader" => links::http::http_header_any,
        "httpmethod" => links::http::http_method_any,
        "mathmlelement" => links::mathmlxref::mathmlxref_any,

        // sidebars
        "apiref" => sidebars::apiref_any,
        "defaultapisidebar" => sidebars::default_api_sidebar_any,
        "jsref" => sidebars::jsref_any,
        "cssref" => sidebars::cssref_any,
        "glossarysidebar" => sidebars::glossarysidebar_any,
        "quicklinkswithsubpages" => quick_links_with_subpages::quick_links_with_subpages_any,
        "addonsidebar" => sidebars::addonsidebar_any,

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
