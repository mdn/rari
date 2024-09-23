use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::helpers::web_ext_examples::{WebExtExample, WEB_EXT_EXAMPLES_DATA};

#[rari_f]
pub fn web_ext_examples(heading: Option<String>) -> Result<String, DocError> {
    let mut split = env.slug.rsplitn(3, '/');
    let leaf = split.next();
    let parent = split.next();
    let examples = match (parent, leaf) {
        (Some("API"), Some(leaf)) => WEB_EXT_EXAMPLES_DATA.by_module.get(leaf),
        (Some(parent), Some(leaf)) => WEB_EXT_EXAMPLES_DATA
            .by_module_and_api
            .get([parent, leaf].join(".").as_str()),
        _ => None,
    };

    if let Some(examples) = examples {
        example_links(examples, heading.as_deref().unwrap_or("h3"), env.locale)
    } else {
        Ok(Default::default())
    }
}

fn example_links(
    examples: &[&WebExtExample],
    heading: &str,
    locale: Locale,
) -> Result<String, DocError> {
    let mut out = String::new();
    if !examples.is_empty() {
        out.extend([
            "<",
            heading,
            ">",
            l10n_json_data("Template", "example_extensions_heading", locale)?,
            "</",
            heading,
            "><ul>",
        ]);
        for example in examples {
            out.extend([
                r#"<li><a href="https://github.com/mdn/webextensions-examples/tree/main/"#,
                example.name.as_str(),
                r#"">"#,
                example.name.as_str(),
                r#"</a></li>"#,
            ]);
        }
        out.push_str("</ul>");
    }
    Ok(out)
}
