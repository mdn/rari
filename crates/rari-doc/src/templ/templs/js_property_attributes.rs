use rari_templ_func::rari_f;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;

#[rari_f(register = "crate::Templ")]
pub fn js_property_attributes(
    writable: i64,
    enumerable: i64,
    configurable: i64,
) -> Result<String, DocError> {
    let writable = writable != 0;
    let enumerable = enumerable != 0;
    let configurable = configurable != 0;
    let yes = l10n_json_data("Template", "yes", env.locale)?;
    let no = l10n_json_data("Template", "no", env.locale)?;

    let mut out = String::new();
    out.extend([
        r#"<table class="standard-table"><thead><tr><th class="header" colspan="2">"#,
        l10n_json_data(
            "Template",
            "js_property_attributes_header_prefix",
            env.locale,
        )?,
        "<code>",
        env.title,
        "</code>",
        l10n_json_data(
            "Template",
            "js_property_attributes_header_suffix",
            env.locale,
        )?,
        r#"</th></tr></thead><tbody><tr><td>"#,
        l10n_json_data("Template", "writable", env.locale)?,
        r#"</td><td>"#,
        if writable { yes } else { no },
        r#"</td></tr><tr><td>"#,
        l10n_json_data("Template", "enumerable", env.locale)?,
        r#"</td><td>"#,
        if enumerable { yes } else { no },
        r#"</td></tr><tr><td>"#,
        l10n_json_data("Template", "configurable", env.locale)?,
        r#"</td><td>"#,
        if configurable { yes } else { no },
        r#"</td></tr></tbody></table>"#,
    ]);
    Ok(out)
}
