use std::fmt::Write;

use css_syntax::syntax::CssType;
use rari_templ_func::rari_f;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::css_info::{
    css_animation_type, css_applies_to, css_computed, css_inherited, css_initial,
    css_l10n_for_value, css_percentages, css_ref_data, css_related_at_rule,
};
use crate::templ::api::RariApi;
use css_syntax_types::{AtRuleDescriptor, Property};

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

    let mut slug_rev_iter = env.slug.rsplitn(3, '/');

    let typ = match env.page_type {
        rari_types::fm_types::PageType::CssAtRuleDescriptor => {
            CssType::AtRuleDescriptor(at_rule.unwrap_or_default(), slug_rev_iter.next().unwrap())
        }
        rari_types::fm_types::PageType::CssProperty => CssType::Property(&name),
        rari_types::fm_types::PageType::CssShorthandProperty => CssType::Property(&name),
        _ => {
            tracing::error!(
                "cssinfo is only valid on properties and at rule descriptors: {}",
                env.slug
            );
            return Err(DocError::CssPageTypeRequired);
        }
    };

    let scope = scope_from_browser_compat(env.browser_compat.first().map(|s| s.as_str()));

    let mut out = String::new();

    match typ {
        CssType::AtRuleDescriptor(at_rule, descriptor) => {
            let rd = get_atrule_descriptor_def(at_rule, descriptor, scope)?;
            render_formal_at_rule_descriptor_def(rd, &mut out, env.locale)?;
        }
        CssType::Property(property) => {
            let pr = get_property_def(property, scope)?;
            render_formal_property_def(pr, &mut out, env.locale)?;
        }
        _ => (),
    }

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
    let atrules = &css_ref_data().atrules;
    let scopes = scope_chain(scope);
    for scope in scopes {
        if let Some(scoped) = atrules.get(scope)
            && let Some(item) = scoped.get(at_rule)
            && let Some(desc) = item.descriptors.get(descriptor)
        {
            return Ok(desc);
        }
    }
    Err(DocError::WebrefLookupFailed)
}

pub fn get_property_def<'a>(property: &str, scope: Option<&str>) -> Result<&'a Property, DocError> {
    let properties = &css_ref_data().properties;
    let scopes = scope_chain(scope);
    for scope in scopes {
        if let Some(scoped) = properties.get(scope)
            && let Some(item) = scoped.get(property)
        {
            return Ok(item);
        }
    }
    Err(DocError::WebrefLookupFailed)
}

fn render_formal_at_rule_descriptor_def(
    at_rule_descriptor: &AtRuleDescriptor,
    out: &mut String,
    locale: Locale,
) -> Result<(), DocError> {
    out.push_str(r#"<table class="properties"><tbody>"#);

    {
        let label = css_related_at_rule(locale)?;
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
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "<code>{value}</code>")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &at_rule_descriptor.computed_value {
        let label = css_computed(locale)?;
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    } else {
        let label = css_computed(locale)?;
        let value = css_l10n_for_value("as specified", locale);
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
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "<code>{value}</code>")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.applies_to {
        let label = css_applies_to(locale)?;
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.inherited {
        let label = css_inherited(locale)?;
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.computed_value {
        let label = css_computed(locale)?;
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.percentages
        && value.to_lowercase() != "n/a"
    {
        let label = css_percentages(locale)?;
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    if let Some(value) = &property.animation_type {
        let label = css_animation_type(locale)?;
        let label = RariApi::link(
            "Web/CSS/Guides/Animations/Animatable_properties",
            Some(locale),
            Some(&label),
            false,
            None,
            false,
        )?;
        let value = css_l10n_for_value(value, locale);
        write!(out, r#"<tr><th scope="row">{label}</th><td>"#)?;
        write!(out, "{value}")?;
        write!(out, r#"</td></tr>"#)?;
    }

    out.push_str(r#"</tbody></table>"#);
    Ok(())
}
