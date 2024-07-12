use std::fmt::Write;

use rari_templ_func::rari_f;
use serde_json::Value;

use crate::error::DocError;
use crate::helpers::css_info::{css_info_properties, mdn_data_files, write_computed_output};

#[rari_f]
pub fn cssinfo() -> Result<String, DocError> {
    let name = env
        .slug
        .rsplit('/')
        .next()
        .map(str::to_lowercase)
        .unwrap_or_default();
    let at_rule = env.slug.strip_prefix("Web/CSS/").and_then(|at_rule| {
        if at_rule.starts_with('@') {
            Some(&at_rule[..at_rule.find('/').unwrap_or(at_rule.len())])
        } else {
            None
        }
    });
    let data = mdn_data_files();
    let css_info_data = if let Some(at_rule) = at_rule {
        &data.css_at_rules.get(at_rule).unwrap_or(&Value::Null)["descriptors"][&name]
    } else {
        data.css_properties.get(&name).unwrap_or(&Value::Null)
    };

    let mut out = String::new();
    out.push_str(r#"<table class="properties"><tbody>"#);
    for (name, label) in css_info_properties(at_rule, env.locale, css_info_data)? {
        write!(&mut out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write_computed_output(env, &mut out, env.locale, css_info_data, name, at_rule)?;
        write!(&mut out, r#"</td></tr>"#)?;
    }
    out.push_str(r#"</tbody></table>"#);
    Ok(out)
}
