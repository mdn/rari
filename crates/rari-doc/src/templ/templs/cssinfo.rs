use std::fmt::Write;

use css_syntax::syntax::CssType;
use css_syntax_types::{AtRuleDescriptor, Property};
use rari_deps::webref_css::css_ref_data;
use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::css_info::{
    css_animation_type, css_applies_to, css_computed, css_inherited, css_initial,
    css_l10n_for_value, css_percentages, css_related_at_rule,
};
use crate::html::links::post_process_templ_links;

const GLOBAL_SCOPE: &str = "__global_scope__";

const PROSE_INITIAL_VALUES: [&str; 6] = [
    "See individual properties",
    "depends on user agent",
    "not defined for shorthand properties",
    "see individual properties",
    "the guaranteed-invalid value (but see prose)",
    "n/a (see prose)",
];

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

    let typ = match env.page_type {
        PageType::CssAtRuleDescriptor => {
            CssType::AtRuleDescriptor(&name, at_rule.unwrap_or_default())
        }
        PageType::CssProperty | PageType::CssShorthandProperty => CssType::Property(&name),
        _ => {
            // Orphaned and conflicting translated pages carry stale macros; don't report them.
            if env.slug.starts_with("orphaned") || env.slug.starts_with("conflicting") {
                return Ok(Default::default());
            }
            tracing::error!(
                "Macro cssinfo can only be used on CSS property and at-rule descriptor pages, but {} is of type {:?}",
                env.slug,
                env.page_type,
            );
            return Err(DocError::CssPageTypeRequired);
        }
    };

    if env.browser_compat.len() > 1 {
        tracing::warn!(
            "Multiple browser-compat entries on {}. The CSS formal definition uses the first one as scope: {}",
            env.slug,
            env.browser_compat[0]
        );
    }
    let scope = scope_from_browser_compat(env.browser_compat.first().map(String::as_str));

    let mut out = String::new();
    match typ {
        CssType::AtRuleDescriptor(descriptor, at_rule) => {
            let def = get_at_rule_descriptor_def(at_rule, descriptor, scope)?;
            render_at_rule_descriptor_def(def, &mut out, env.locale)?;
        }
        CssType::Property(property) => {
            let def = get_property_def(property, scope)?;
            render_property_def(def, &mut out, env.locale)?;
        }
        _ => unreachable!(),
    }

    post_process_templ_links(&out)
}

/// Extracts the webref scope (the feature name) from a BCD key like `css.properties.text-align`.
fn scope_from_browser_compat(browser_compat: Option<&str>) -> Option<&str> {
    browser_compat.and_then(|bc| bc.split('.').nth(2))
}

fn scope_chain(scope: Option<&str>) -> impl Iterator<Item = &str> {
    scope.into_iter().chain([GLOBAL_SCOPE])
}

fn get_at_rule_descriptor_def<'a>(
    at_rule: &str,
    descriptor: &str,
    scope: Option<&str>,
) -> Result<&'a AtRuleDescriptor, DocError> {
    let at_rules = &css_ref_data().atrules;
    scope_chain(scope)
        .find_map(|scope| {
            at_rules
                .get(scope)?
                .get(at_rule)?
                .descriptors
                .get(descriptor)
        })
        .ok_or_else(|| {
            DocError::WebrefLookupFailed(format!(
                "descriptor '{descriptor}' of at-rule '{at_rule}' not found"
            ))
        })
}

fn get_property_def<'a>(property: &str, scope: Option<&str>) -> Result<&'a Property, DocError> {
    let properties = &css_ref_data().properties;
    scope_chain(scope)
        .find_map(|scope| properties.get(scope)?.get(property))
        .ok_or_else(|| DocError::WebrefLookupFailed(format!("property '{property}' not found")))
}

fn write_table_row(out: &mut String, label: &str, value: &str) -> Result<(), DocError> {
    Ok(write!(
        out,
        r#"<tr><th scope="row">{label}</th><td>{value}</td></tr>"#
    )?)
}

fn escape_css_value(value: &str) -> String {
    html_escape::encode_text(value).into_owned()
}

fn render_css_value(value: &str, locale: Locale) -> String {
    escape_css_value(css_l10n_for_value(value, locale))
}

fn is_prose_initial_value(value: &str) -> bool {
    PROSE_INITIAL_VALUES.contains(&value)
}

fn render_initial_value(value: &str, locale: Locale) -> String {
    let is_prose = is_prose_initial_value(value);
    let value = render_css_value(value, locale);
    if is_prose {
        value
    } else {
        format!("<code>{value}</code>")
    }
}

fn render_at_rule_descriptor_def(
    descriptor: &AtRuleDescriptor,
    out: &mut String,
    locale: Locale,
) -> Result<(), DocError> {
    out.push_str(r#"<table class="properties"><tbody>"#);

    let at_rule = &descriptor.r#for;
    write_table_row(
        out,
        &css_related_at_rule(locale)?,
        &format!(
            r#"<a href="/{}/docs/Web/CSS/Reference/At-rules/{at_rule}"><code>{at_rule}</code></a>"#,
            locale.as_url_str(),
        ),
    )?;

    if let Some(value) = &descriptor.initial {
        write_table_row(
            out,
            &css_initial(locale)?,
            &render_initial_value(value, locale),
        )?;
    }

    let computed = descriptor
        .computed_value
        .as_deref()
        .unwrap_or("as specified");
    write_table_row(
        out,
        &css_computed(locale)?,
        &render_css_value(computed, locale),
    )?;

    out.push_str(r#"</tbody></table>"#);
    Ok(())
}

fn render_property_def(
    property: &Property,
    out: &mut String,
    locale: Locale,
) -> Result<(), DocError> {
    out.push_str(r#"<table class="properties"><tbody>"#);

    if let Some(value) = &property.initial {
        write_table_row(
            out,
            &css_initial(locale)?,
            &render_initial_value(value, locale),
        )?;
    }
    if let Some(value) = &property.applies_to {
        write_table_row(
            out,
            &css_applies_to(locale)?,
            &render_css_value(value, locale),
        )?;
    }
    if let Some(value) = &property.inherited {
        write_table_row(
            out,
            &css_inherited(locale)?,
            &render_css_value(value, locale),
        )?;
    }
    if let Some(value) = &property.computed_value {
        write_table_row(
            out,
            &css_computed(locale)?,
            &render_css_value(value, locale),
        )?;
    }
    if let Some(value) = &property.percentages
        && !value.eq_ignore_ascii_case("n/a")
    {
        write_table_row(
            out,
            &css_percentages(locale)?,
            &render_css_value(value, locale),
        )?;
    }
    if let Some(value) = &property.animation_type {
        write_table_row(
            out,
            &css_animation_type(locale)?,
            &render_css_value(value, locale),
        )?;
    }

    out.push_str(r#"</tbody></table>"#);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_prose_initial_values() {
        let cases = [
            ("lowercase", "see individual properties", true),
            ("capitalized", "See individual properties", true),
            ("user agent", "depends on user agent", true),
            ("shorthand", "not defined for shorthand properties", true),
            ("prose reference", "n/a (see prose)", true),
            (
                "guaranteed invalid",
                "the guaranteed-invalid value (but see prose)",
                true,
            ),
            ("keyword", "auto", false),
            ("multiple CSS tokens", "0 1 auto", false),
        ];

        for (name, value, expected) in cases {
            assert_eq!(is_prose_initial_value(value), expected, "{name}");
        }
    }

    #[test]
    fn renders_prose_initial_values_without_code() {
        assert_eq!(
            render_initial_value("see individual properties", Locale::EnUs),
            "see individual properties"
        );
        assert_eq!(
            render_initial_value("auto", Locale::EnUs),
            "<code>auto</code>"
        );
    }

    #[test]
    fn test_scope_from_browser_compat() {
        let cases = [
            (
                "property key",
                Some("css.properties.text-align"),
                Some("text-align"),
            ),
            (
                "descriptor key",
                Some("css.at-rules.font-face.src"),
                Some("font-face"),
            ),
            ("too short", Some("css.properties"), None),
            ("none", None, None),
        ];
        for (name, input, expected) in cases {
            assert_eq!(scope_from_browser_compat(input), expected, "{name}");
        }
    }

    #[test]
    fn test_get_property_def() {
        let cases = [
            ("global scope", "color", None, true),
            ("scoped", "color", Some("color"), true),
            (
                "unknown scope falls back to global",
                "color",
                Some("nope"),
                true,
            ),
            ("unknown property", "not-a-property", None, false),
        ];
        for (name, property, scope, found) in cases {
            assert_eq!(get_property_def(property, scope).is_ok(), found, "{name}");
        }
    }

    #[test]
    fn test_get_at_rule_descriptor_def() {
        let cases = [
            ("global scope", "@font-face", "src", None, true),
            ("scoped", "@font-face", "src", Some("font-face"), true),
            ("unknown descriptor", "@font-face", "nope", None, false),
            ("unknown at-rule", "@nope", "src", None, false),
        ];
        for (name, at_rule, descriptor, scope, found) in cases {
            assert_eq!(
                get_at_rule_descriptor_def(at_rule, descriptor, scope).is_ok(),
                found,
                "{name}"
            );
        }
    }

    #[test]
    fn test_render_css_value_as_html() -> Result<(), Box<dyn std::error::Error>> {
        let grammar_placeholders = "<angle> <basic-shape> <color> <custom-ident> <display-box> <image> <integer> <length> <number> <percentage> <resolution> <string> <time> <transform-function> <url>";
        let escaped = escape_css_value(grammar_placeholders);
        let html = rari_md::m2h(
            &format!("<table><tr><td>{escaped}</td></tr></table>"),
            Locale::EnUs,
        )?;

        assert_eq!(
            html,
            "<table><tr><td>&lt;angle&gt; &lt;basic-shape&gt; &lt;color&gt; &lt;custom-ident&gt; &lt;display-box&gt; &lt;image&gt; &lt;integer&gt; &lt;length&gt; &lt;number&gt; &lt;percentage&gt; &lt;resolution&gt; &lt;string&gt; &lt;time&gt; &lt;transform-function&gt; &lt;url&gt;</td></tr></table>\n"
        );
        Ok(())
    }
}
