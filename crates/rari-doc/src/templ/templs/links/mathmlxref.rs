use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::templ::element_xref::{ElementFamily, link_element};

/// Creates a link to a MathML element reference page on MDN.
///
/// This macro generates links to MathML element documentation. It automatically
/// formats the display text with angle brackets (e.g., `<math>`) and applies
/// code formatting to distinguish MathML elements in the text. The element name
/// is converted to lowercase for consistency.
///
/// # Arguments
/// * `element_name` - The MathML element name (will be converted to lowercase)
///
/// # Examples
/// * `{{MathMLElement("math")}}` -> links to `<math>` element with code formatting
/// * `{{MathMLElement("mrow")}}` -> links to `<mrow>` element
/// * `{{MathMLElement("mi")}}` -> links to `<mi>` element for identifiers
/// * `{{MathMLElement("mo")}}` -> links to `<mo>` element for operators
///
/// # Special handling
/// - Converts element name to lowercase for URL consistency
/// - Automatically wraps element name in `&lt;` and `&gt;` for display
/// - Sets title attribute with unescaped angle brackets for accessibility
/// - Uses code formatting (`<code>` tags) by default
/// - Links to `/Web/MathML/Reference/Element/{element_name}` path structure
#[rari_f(register = "crate::Templ")]
pub fn mathmlelement(element_name: String) -> Result<String, DocError> {
    let element_name = element_name.to_lowercase();
    let display = format!("&lt;{element_name}&gt;");
    let title = format!("<{element_name}>");
    link_element(
        ElementFamily::Mathml,
        &element_name,
        &display,
        true,
        Some(&title),
        None,
        env.locale,
    )
}
