use rari_templ_func::rari_f;
use rari_types::globals::{json_svg_data_lookup, SVGDataDescription};

use super::links::svgxref::svgxref_internal;
use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;

#[rari_f]
pub fn svginfo() -> Result<String, DocError> {
    let name = env
        .slug
        .rsplit_once('/')
        .map(|(_, name)| name)
        .ok_or_else(|| DocError::InvalidSlugForX(env.slug.to_string()))?;
    let info = json_svg_data_lookup().get(name);

    let mut out = String::new();
    if let Some(info) = info {
        out.extend([
            r#"<table class="properties"><tbody><tr><th scope="row">"#,
            l10n_json_data("SVG", "categories", env.locale)?,
            r#"</th><td>"#,
        ]);
        out.extend(itertools::intersperse(
            info.categories.iter().map(|category| {
                l10n_json_data("SVG", category, env.locale).unwrap_or(category.as_str())
            }),
            l10n_json_data("Common", "listSeparator", env.locale).unwrap_or(", "),
        ));
        out.push_str(r#"</td></tr>"#);

        let (element_groups, elements): (Vec<String>, Vec<String>) = info
            .content
            .elements
            .iter()
            .try_fold(
            (vec![], vec![]),
            |(mut element_groups, mut elements), element| {
                if element.contains("&lt;") {
                    let element_name = element.strip_prefix("&lt;").unwrap_or(element);
                    let element_name = element_name.strip_suffix("&gt;").unwrap_or(element_name);
                    elements.push(svgxref_internal(element_name, env.locale)?)
                } else {
                    let anchor = to_snake_case(element);
                    let url = format!("/{}/docs/Web/SVG/Element#{anchor}", env.locale.as_url_str());
                    let display = l10n_json_data("SVG", element, env.locale).unwrap_or(element);
                    let link = RariApi::link(&url, env.locale, Some(display), false, None, false)?;
                    element_groups.push(link);
                }
                Ok::<_, DocError>((element_groups, elements))
            },
        )?;

        out.extend([
            r#"<tr><th scope="row">"#,
            l10n_json_data("SVG", "permittedContent", env.locale)?,
            r#"</th><td>"#,
            match info.content.description {
                SVGDataDescription::Copy(ref s) => l10n_json_data("SVG", s, env.locale)?,
                SVGDataDescription::L10n(ref map) => map
                    .get(&env.locale)
                    .or(map.get(&Default::default()))
                    .map(|s| s.as_str())
                    .unwrap_or_default(),
            },
        ]);

        if !element_groups.is_empty() {
            out.push_str("<br/>");
            out.extend(itertools::intersperse(
                element_groups.iter().map(|s| s.as_str()),
                "<br/>",
            ))
        }
        if !elements.is_empty() {
            out.push_str("<br/>");
            out.extend(itertools::intersperse(
                elements.iter().map(|s| s.as_str()),
                l10n_json_data("Common", "listSeparator", env.locale).unwrap_or(", "),
            ))
        }

        out.push_str(r#"</td></tr></tbody></table>"#);
    } else {
        return Err(DocError::InvalidTempl(format!("No svginfor for {name}")));
    }

    Ok(out)
}

fn to_snake_case(s: &str) -> String {
    let underscore_count = s
        .chars()
        .skip(1)
        .filter(|&c| c.is_ascii_uppercase())
        .count();
    let mut result = String::with_capacity(s.len() + underscore_count);

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}
