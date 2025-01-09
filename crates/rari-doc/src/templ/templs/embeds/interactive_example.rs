use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;

/// Adds an <interactive-example> element to the content
///
/// Parameters:
///  $0 - Optional custom height class to set on interactive-example element
///
///  Example call {{InteractiveExample("taller")}}
#[rari_f]
pub fn interactive_example(height: Option<String>) -> Result<String, DocError> {
    let title = l10n_json_data("Template", "interactive_example_cta", env.locale)?;
    let id = RariApi::anchorize(title);
    let h = height.unwrap_or_default();
    Ok(format!(
        r#"<h2 id="{id}">{title}</h2>
<interactive-example height="{h}"></interactive-example>"#
    ))
}
