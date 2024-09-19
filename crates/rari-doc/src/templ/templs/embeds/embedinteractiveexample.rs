use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;

/// Embeds a live sample from a interactive-examples.mdn.mozilla.net GitHub page
///
/// Parameters:
///  $0 - The URL of interactive-examples.mdn.mozilla.net page (relative)
///  $1 - Optional custom height class to set on iframe element
///
///  Example call {{EmbedInteractiveExample("pages/css/animation.html", "taller")}}
#[rari_f]
pub fn embed_interactive_example(path: String, height: Option<String>) -> Result<String, DocError> {
    let title = l10n_json_data("Template", "interactive_example_cta", env.locale)?;
    let url = format!("{}/{path}", RariApi::interactive_examples_base_url());
    let height_class = match height.as_deref() {
        h @ Some("shorter" | "taller" | "tabbed-shorter" | "tabbed-standard" | "tabbed-taller") => {
            h.unwrap()
        }
        None if path.contains("/js/") => "js",
        None | Some(_) => "default",
    };
    let id = RariApi::anchorize(title);
    Ok(format!(
        r#"<h2 id="{id}">{title}</h2>
<iframe class="interactive is-{height_class}-height" height="200" src="{url}" title="MDN Web Docs Interactive Example" allow="clipboard-write"></iframe>"#
    ))
}
