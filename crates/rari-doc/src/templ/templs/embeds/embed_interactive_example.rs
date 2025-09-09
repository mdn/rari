use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;

/// Embeds an interactive example from the MDN interactive examples repository.
///
/// This macro creates an iframe that displays interactive code examples from the
/// MDN interactive-examples repository, allowing users to experiment with code
/// directly in the browser. It automatically determines appropriate height classes
/// based on the content type and generates localized section headings.
///
/// # Arguments
/// * `path` - Relative path to the interactive example (e.g., "pages/css/animation.html")
/// * `height` - Optional height class for the iframe ("shorter", "taller", "tabbed-shorter", "tabbed-standard", "tabbed-taller")
///
/// # Examples
/// * `{{EmbedInteractiveExample("pages/css/animation.html")}}` -> embeds CSS animation example with default height
/// * `{{EmbedInteractiveExample("pages/js/array-map.html", "taller")}}` -> JavaScript example with taller height
/// * `{{EmbedInteractiveExample("pages/css/flexbox.html", "tabbed-standard")}}` -> CSS example with tabbed interface
///
/// # Special handling
/// - Automatically uses "js" height class for JavaScript examples (paths containing "/js/")
/// - Falls back to "default" height class if no valid height specified
/// - Generates localized "Try it" heading with proper anchor ID
/// - Links to interactive-examples.mdn.mozilla.net with clipboard-write permissions
/// - Creates accessible iframe with descriptive title
#[rari_f(register = "crate::Templ")]
pub fn embedinteractiveexample(path: String, height: Option<String>) -> Result<String, DocError> {
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
