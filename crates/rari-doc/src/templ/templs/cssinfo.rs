use std::fmt::Write;
use std::fs;
use std::sync::LazyLock;

use css_syntax::syntax::CssType;
use rari_templ_func::rari_f;
use rari_types::globals::data_dir;
use rari_types::locale::Locale;
use serde_json::Value;

use crate::error::DocError;
use crate::helpers::css_info::{
    css_computed, css_info_properties, css_inherited, css_initial, get_css_l10n_for_locale,
    mdn_data_files, write_computed_output, write_missing,
};
use crate::helpers::l10n::l10n_json_data;
use crate::templ::api::RariApi;
use css_syntax_types::WebrefCss;
use css_syntax_types::{AtRuleDescriptor, Property};

static CSS_REF: LazyLock<WebrefCss> = LazyLock::new(|| {
    let json_str = fs::read_to_string(data_dir().join("@webref/css").join("webref_css.json"))
        .expect("no data dir");
    serde_json::from_str(&json_str).expect("Failed to parse JSON")
});

#[rari_f(register = "crate::Templ")]
pub fn cssinfo() -> Result<String, DocError> {
    let name = env
        .slug
        .rsplit('/')
        .next()
        .map(str::to_lowercase)
        .unwrap_or_default();

    let at_rule = env
        .slug
        .strip_prefix("Web/CSS/Reference/At-rules/")
        .and_then(|at_rule| {
            if at_rule.starts_with('@') {
                Some(&at_rule[..at_rule.find('/').unwrap_or(at_rule.len())])
            } else {
                None
            }
        });

    println!("== CSSINFO env {:#?}", env);

    let key = "all elements except inline boxes";
    let term = l10n(key, env.locale);

    println!("== CSSINFO term {term}");

    // get data from webref
    let mut slug_rev_iter = env.slug.rsplitn(3, '/');

    let typ = match env.page_type {
        rari_types::fm_types::PageType::CssAtRuleDescriptor => {
            CssType::AtRuleDescriptor(at_rule.unwrap_or_default(), slug_rev_iter.next().unwrap())
        }
        rari_types::fm_types::PageType::CssProperty => CssType::Property(&name),
        _ => {
            tracing::error!(
                "cssinfo is only valid on properties and at rule descriptors: {}",
                env.slug
            );
            return Err(DocError::CssPageTypeRequired);
        }
    };

    println!("== CSSINFO typ {typ:?}");
    let scope = scope_from_browser_compat(env.browser_compat.first().map(|s| s.as_str()));
    println!("== CSSINFO scope {scope:?}");

    let mut out = String::new();

    match typ {
        CssType::AtRuleDescriptor(at_rule, descriptor) => {
            println!("== CSSINFO at rule descriptor {at_rule} {descriptor}");
            let rd = get_atrule_descriptor_def(at_rule, descriptor, scope)?;
            render_formal_at_rule_descriptor_def(rd, &mut out, env.locale)?;
            println!("== CSSINFO rendered at rule description: {out}");
        }
        CssType::Property(property) => {
            println!("== CSSINFO property {property}");
            let pr = get_property_def(property, scope)?;
            render_formal_property_def(pr, &mut out, env.locale)?;
            println!("== CSSINFO rendered property: {out}");
        }
        _ => (),
    }

    // =========================

    let data = mdn_data_files();
    let css_info_data = if let Some(at_rule) = at_rule {
        &data.css_at_rules.get(at_rule).unwrap_or(&Value::Null)["descriptors"][&name]
    } else {
        data.css_properties.get(&name).unwrap_or(&Value::Null)
    };
    let props = css_info_properties(at_rule, env.locale, css_info_data)?;

    if props.is_empty() {
        write_missing(&mut out, env.locale)?;
        return Ok(out);
    }
    out.push_str(r#"<table class="properties"><tbody>"#);
    for (name, label) in props {
        write!(&mut out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write_computed_output(env, &mut out, env.locale, css_info_data, name, at_rule)?;
        write!(&mut out, r#"</td></tr>"#)?;
    }
    out.push_str(r#"</tbody></table>"#);
    Ok(out)
}

fn scope_from_browser_compat(browser_compat: Option<&str>) -> Option<&str> {
    if let Some(bc) = browser_compat {
        bc.split(".").collect::<Vec<&str>>().get(2).copied()
    } else {
        None
    }
}

fn scope_chain(scope: Option<&str>) -> Vec<&str> {
    scope.into_iter().chain(["__global_scope__"]).collect()
}

fn get_atrule_descriptor_def<'a>(
    at_rule: &str,
    descriptor: &str,
    scope: Option<&'a str>,
) -> Result<&'a AtRuleDescriptor, DocError> {
    let atrules = &CSS_REF.atrules;
    let scopes = scope_chain(scope);
    for scope in scopes {
        if let Some(scoped) = atrules.get(scope)
            && let Some(item) = scoped.get(at_rule)
            && let Some(desc) = item.descriptors.get(descriptor)
        {
            println!(
                "== CSSINFO Found descriptor {} for {} in scope {}: {:#?}",
                descriptor, at_rule, scope, desc
            );
            return Ok(desc);
        }
    }
    Err(DocError::WebrefLookupFailed)
}

fn get_property_def<'a>(property: &str, scope: Option<&str>) -> Result<&'a Property, DocError> {
    let properties = &CSS_REF.properties;
    let scopes = scope_chain(scope);
    for scope in scopes {
        if let Some(scoped) = properties.get(scope)
            && let Some(item) = scoped.get(property)
        {
            println!(
                "== CSSINFO Found property {} in scope {}: {:#?}",
                property, scope, item
            );
            return Ok(item);
        }
    }
    Err(DocError::WebrefLookupFailed)
}

fn l10n(key: &str, locale: Locale) -> &str {
    l10n_json_data(
        "CSSFormalDefinitions",
        key,
        locale,
    )
    .inspect_err(|e| tracing::warn!("Localized value for formal definition is missing in content/files/jsondata/L10n-CSSFormalDefinitions.json: {} ({})", key, e))
    .unwrap_or(key)
}

fn render_formal_at_rule_descriptor_def(
    at_rule_descriptor: &AtRuleDescriptor,
    out: &mut String,
    locale: Locale,
) -> Result<(), DocError> {
    out.push_str(r#"<table class="properties"><tbody>"#);

    {
        let label = get_css_l10n_for_locale("relatedAtRule", locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(
            out,
            r#"<a href="/{}/docs/Web/CSS/Reference/At-rules/{}"><code>{}</code></a>"#,
            locale.as_url_str(),
            &at_rule_descriptor.r#for,
            &at_rule_descriptor.r#for
        )?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &at_rule_descriptor.initial {
        let label = css_initial(locale)?;
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "<code>{value}</code>")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &at_rule_descriptor.computed_value {
        let label = css_computed(locale)?;
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    out.push_str(r#"</tbody></table>"#);
    Ok(())
}

fn render_formal_property_def(
    property: &Property,
    out: &mut String,
    locale: Locale,
) -> Result<(), DocError> {
    out.push_str(r#"<table class="properties"><tbody>"#);

    if let Some(value) = &property.initial {
        let label = css_initial(locale)?;
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "<code>{value}</code>")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.applies_to {
        let label = get_css_l10n_for_locale("appliesTo", locale);
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.inherited {
        let label = css_inherited(locale)?;
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.computed_value {
        let label = css_computed(locale)?;
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.percentages
        && value.to_lowercase() != "n/a"
    {
        let label = get_css_l10n_for_locale("percentages", locale);
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.animation_type {
        let label = get_css_l10n_for_locale("animationType", locale);
        let label = RariApi::link(
            "Web/CSS/Guides/Animations/Animatable_properties",
            Some(locale),
            Some(label),
            false,
            None,
            false,
        )?;
        let value = l10n(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    out.push_str(r#"</tbody></table>"#);
    Ok(())
}
