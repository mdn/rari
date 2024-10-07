pub mod api_list_alpha;
pub mod api_list_specs;
pub mod badges;
pub mod banners;
pub mod compat;
pub mod css_ref;
pub mod cssinfo;
pub mod csssyntax;
pub mod echo;
pub mod embeds;
pub mod firefox_for_developers;
pub mod glossary;
pub mod glossarydisambiguation;
pub mod inheritance_diagram;
pub mod inline_labels;
pub mod js_property_attributes;
pub mod links;
pub mod list_subpages_for_sidebar;
pub mod listsubpages;
pub mod previous_menu_next;
pub mod quick_links_with_subpages;
pub mod sidebars;
pub mod specification;
pub mod subpages_with_summaries;
pub mod svginfo;
pub mod web_ext_examples;
pub mod webext_all_examples;
pub mod xsltref;

use rari_types::globals::deny_warnings;
use rari_types::{Arg, RariEnv};
use tracing::error;

use crate::error::DocError;
use crate::utils::TEMPL_RECORDER;

pub fn invoke(
    env: &RariEnv,
    name: &str,
    args: Vec<Option<Arg>>,
) -> Result<(String, bool), DocError> {
    // TODO: improve sidebar handling
    let is_sidebar = matches!(
        name,
        "apiref"
            | "defaultapisidebar"
            | "jsref"
            | "cssref"
            | "glossarysidebar"
            | "quicklinkswithsubpages"
            | "addonsidebar"
            | "learnsidebar"
            | "svgref"
            | "httpsidebar"
            | "jssidebar"
            | "htmlsidebar"
            | "accessibilitysidebar"
            | "firefoxsidebar"
            | "webassemblysidebar"
            | "xsltsidebar"
            | "mdnsidebar"
            | "gamessidebar"
            | "mathmlref"
            | "pwasidebar"
            | "addonsidebarmain"
    );
    let f = match name {
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
        "xsltref" => xsltref::xsltref_any,
        "webextallcompattables" => compat::webextallcompattables_any,
        "webextallexamples" => webext_all_examples::web_ext_all_examples_any,
        "listgroups" => api_list_specs::api_list_specs_any,
        "apilistalpha" => api_list_alpha::api_list_alpha_any,
        "css_ref" => css_ref::css_ref_any,

        // hacky
        "glossarydisambiguation" => glossarydisambiguation::glossarydisambiguation_any,
        "listsubpagesforsidebar" => list_subpages_for_sidebar::list_subpages_for_sidebar_any,
        "subpageswithsummaries" | "landingpagelistsubpages" => {
            subpages_with_summaries::subpages_with_summaries_any
        }

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
        "learnsidebar" => sidebars::learnsidebar_any,
        "svgref" => sidebars::svgref_any,
        "httpsidebar" => sidebars::httpsidebar_any,
        "jssidebar" => sidebars::jssidebar_any,
        "htmlsidebar" => sidebars::htmlsidebar_any,
        "accessibilitysidebar" => sidebars::accessibilitysidebar_any,
        "firefoxsidebar" => sidebars::firefoxsidebar_any,
        "webassemblysidebar" => sidebars::webassemblysidebar_any,
        "xsltsidebar" => sidebars::xsltsidebar_any,
        "mdnsidebar" => sidebars::mdnsidebar_any,
        "gamessidebar" => sidebars::gamessidebar_any,
        "mathmlref" => sidebars::mathmlref_any,
        "pwasidebar" => sidebars::pwasidebar_any,
        "addonsidebarmain" => sidebars::addonsidebarmain_any,

        // ignore
        "xulelem" => return Ok((Default::default(), false)),

        // debug
        "echo" => echo::echo_any,

        // unknown
        _ if deny_warnings() => return Err(DocError::UnknownMacro(name.to_string())),
        _ => {
            TEMPL_RECORDER.with(|tx| {
                if let Some(tx) = tx {
                    if let Err(e) = tx.send(name.to_string()) {
                        error!("templ recorder: {e}");
                    }
                }
            });
            return Ok((format!("<s>unsupported templ: {name}</s>"), is_sidebar));
        } //
    };
    f(env, args).map(|s| (s, is_sidebar))
}
