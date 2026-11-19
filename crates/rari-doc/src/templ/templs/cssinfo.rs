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

const SEE_INDIVIDUAL: &str = "see individual properties";
const MAX_LONGHAND_DEPTH: usize = 4;

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
            render_property_def(def, scope, &mut out, env.locale)?;
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

/// A property's value for one formal definition field, expanded into the
/// values of its longhands where the shorthand defers to them.
struct ValueNode<'a> {
    name: &'a str,
    value: Option<&'a str>,
    longhands: Vec<ValueNode<'a>>,
}

impl<'a> ValueNode<'a> {
    fn new(
        property: &'a Property,
        field: fn(&Property) -> Option<&str>,
        scope: Option<&str>,
        depth: usize,
    ) -> Self {
        let value = field(property);
        let defers_to_longhands = value.is_none_or(|v| v.eq_ignore_ascii_case(SEE_INDIVIDUAL));
        let longhands = if defers_to_longhands && depth < MAX_LONGHAND_DEPTH {
            property
                .longhands
                .iter()
                .filter_map(|name| match get_property_def(name, scope) {
                    Ok(longhand) => Some(Self::new(longhand, field, scope, depth + 1)),
                    Err(e) => {
                        tracing::warn!("Longhand of '{}' not found: {e}", property.name);
                        None
                    }
                })
                .filter(|node| node.value.is_some() || !node.longhands.is_empty())
                .collect()
        } else {
            vec![]
        };
        Self {
            name: &property.name,
            value,
            longhands,
        }
    }

    fn leaf_values(&self, out: &mut Vec<&'a str>) {
        if self.longhands.is_empty() {
            out.extend(self.value);
        } else {
            for longhand in &self.longhands {
                longhand.leaf_values(out);
            }
        }
    }

    /// The single value shared by this property and all its longhands, if any.
    fn uniform_value(&self) -> Option<&'a str> {
        let mut values = vec![];
        self.leaf_values(&mut values);
        let first = *values.first()?;
        values.iter().all(|v| *v == first).then_some(first)
    }

    /// Renders the value, listing longhands unless `merge_equal` is set and they agree.
    /// Nested shorthands whose longhands agree are always collapsed.
    fn render(
        &self,
        merge_equal: bool,
        render_leaf: &dyn Fn(&str) -> String,
        locale: Locale,
    ) -> Option<String> {
        if self.longhands.is_empty() {
            return self.value.map(render_leaf);
        }
        if merge_equal && let Some(value) = self.uniform_value() {
            return Some(render_leaf(value));
        }
        let mut out = String::new();
        out.push_str(&render_css_value(SEE_INDIVIDUAL, locale));
        out.push_str("<br><ul>");
        for longhand in &self.longhands {
            let rendered = longhand.render(true, render_leaf, locale)?;
            let name = longhand.name;
            write!(
                out,
                r#"<li><a href="/{}/docs/Web/CSS/Reference/Properties/{name}"><code>{name}</code></a>: {rendered}</li>"#,
                locale.as_url_str(),
            )
            .ok()?;
        }
        out.push_str("</ul>");
        Some(out)
    }
}

fn render_property_def(
    property: &Property,
    scope: Option<&str>,
    out: &mut String,
    locale: Locale,
) -> Result<(), DocError> {
    out.push_str(r#"<table class="properties"><tbody>"#);

    let css_value = |value: &str| render_css_value(value, locale);
    let initial = |value: &str| render_initial_value(value, locale);

    // Initial value, computed value and animation type always list the longhands
    // individually, the others only if the longhands differ.
    struct Row<'a> {
        label: String,
        field: fn(&Property) -> Option<&str>,
        merge_equal: bool,
        omit_na: bool,
        render_leaf: &'a dyn Fn(&str) -> String,
    }
    let rows = [
        Row {
            label: css_initial(locale)?,
            field: |p| p.initial.as_deref(),
            merge_equal: false,
            omit_na: false,
            render_leaf: &initial,
        },
        Row {
            label: css_applies_to(locale)?,
            field: |p| p.applies_to.as_deref(),
            merge_equal: true,
            omit_na: false,
            render_leaf: &css_value,
        },
        Row {
            label: css_inherited(locale)?,
            field: |p| p.inherited.as_deref(),
            merge_equal: true,
            omit_na: false,
            render_leaf: &css_value,
        },
        Row {
            label: css_computed(locale)?,
            field: |p| p.computed_value.as_deref(),
            merge_equal: false,
            omit_na: false,
            render_leaf: &css_value,
        },
        Row {
            label: css_percentages(locale)?,
            field: |p| p.percentages.as_deref(),
            merge_equal: true,
            omit_na: true,
            render_leaf: &css_value,
        },
        Row {
            label: css_animation_type(locale)?,
            field: |p| p.animation_type.as_deref(),
            merge_equal: false,
            omit_na: false,
            render_leaf: &css_value,
        },
    ];

    for row in rows {
        let node = ValueNode::new(property, row.field, scope, 0);
        if row.omit_na
            && node
                .uniform_value()
                .is_some_and(|v| v.eq_ignore_ascii_case("n/a"))
        {
            continue;
        }
        if let Some(value) = node.render(row.merge_equal, row.render_leaf, locale) {
            write_table_row(out, &row.label, &value)?;
        }
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

    fn render_field(
        property: &str,
        field: fn(&Property) -> Option<&str>,
        merge_equal: bool,
    ) -> Option<String> {
        let scope = Some(property);
        let def = get_property_def(property, scope).unwrap();
        ValueNode::new(def, field, scope, 0).render(
            merge_equal,
            &|value| render_css_value(value, Locale::EnUs),
            Locale::EnUs,
        )
    }

    fn longhand_item(name: &str) -> String {
        format!(
            r#"<li><a href="/en-US/docs/Web/CSS/Reference/Properties/{name}"><code>{name}</code></a>: "#
        )
    }

    #[test]
    fn test_render_shorthand_lists_longhands_recursively() {
        let html = render_field("rule", |p| p.initial.as_deref(), false).unwrap();
        let nested = "see individual properties<br><ul>";
        for (name, needle) in [
            (
                "nested shorthand",
                format!("{}{nested}", longhand_item("column-rule")),
            ),
            (
                "longhand",
                format!("{}medium</li>", longhand_item("column-rule-width")),
            ),
            (
                "sibling longhand",
                format!("{}none</li>", longhand_item("row-rule-style")),
            ),
        ] {
            assert!(html.contains(&needle), "{name}: {html}");
        }
    }

    #[test]
    fn test_render_collapses_uniform_nested_shorthands() {
        let html = render_field("border", |p| p.initial.as_deref(), false).unwrap();
        for (name, needle) in [
            (
                "width",
                format!("{}medium</li>", longhand_item("border-width")),
            ),
            (
                "style",
                format!("{}none</li>", longhand_item("border-style")),
            ),
            (
                "color",
                format!("{}currentcolor</li>", longhand_item("border-color")),
            ),
        ] {
            assert!(html.contains(&needle), "{name}: {html}");
        }
        assert!(!html.contains("border-top-width"), "{html}");
    }

    #[test]
    fn test_render_shorthand_merges_equal_values() {
        struct Case {
            name: &'static str,
            property: &'static str,
            field: fn(&Property) -> Option<&str>,
            merge_equal: bool,
            expect_list: bool,
        }
        let cases = [
            Case {
                name: "equal values merge",
                property: "border-width",
                field: |p| p.inherited.as_deref(),
                merge_equal: true,
                expect_list: false,
            },
            Case {
                name: "equal values listed when not merging",
                property: "border-width",
                field: |p| p.inherited.as_deref(),
                merge_equal: false,
                expect_list: true,
            },
            Case {
                name: "differing values listed",
                property: "border",
                field: |p| p.computed_value.as_deref(),
                merge_equal: true,
                expect_list: true,
            },
        ];
        for case in cases {
            let html = render_field(case.property, case.field, case.merge_equal).unwrap();
            assert_eq!(
                html.contains("<ul>"),
                case.expect_list,
                "{}: {html}",
                case.name
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
