use rari_templ_func::rari_f;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::issues::get_issue_counter;
use crate::pages::page::PageLike;
use crate::templ::api::RariApi;
use crate::templ::css_feature_index::resolve_css_feature;

/// Creates a link to a CSS reference page on MDN.
///
/// This macro generates links to CSS properties, functions, data types, selectors,
/// and other CSS reference documentation. It automatically formats the display text
/// based on the CSS feature type and handles various CSS naming conventions.
///
/// # Arguments
/// * `name` - The CSS feature name (property, function, data type, etc.)
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
///
/// # Examples
/// * `{{CSSxRef("color")}}` -> links to CSS property at `/Web/CSS/Reference/Properties/color`
/// * `{{CSSxRef("background-color", "background color")}}` -> custom display text
/// * `{{CSSxRef("calc()", "", "#syntax")}}` -> links to calc() function with anchor at `/Web/CSS/Reference/Values/calc#syntax`
/// * `{{CSSxRef("&lt;color&gt;")}}` -> links to color data type at `/Web/CSS/Reference/Values/color_value`
/// * `{{CSSxRef(":hover")}}` -> links to pseudo-class at `/Web/CSS/Reference/Selectors/:hover`
/// * `{{CSSxRef("@media")}}` -> links to at-rule at `/Web/CSS/Reference/At-rules/@media`
/// * `{{CSSxRef("@media/color")}}` -> links to color media feature at `/Web/CSS/Reference/At-rules/@media/color`
///
/// # URL Structure
/// The macro resolves the (normalized) name against an index of all
/// `Web/CSS/*` pages (see [`crate::templ::css_feature_index`]). The input
/// syntax narrows the category looked up:
/// - Data types (`<...>` or `&lt;...&gt;`): `Reference/Values/{slug}`
/// - Pseudo-classes/elements (`:...`): `Reference/Selectors/{slug}`
/// - At-rules (`@...`, optionally `/descriptor`): `Reference/At-rules/{slug}`
/// - Functions (`...()`): `Reference/Values/{slug}`
/// - Bare names: `Reference/Properties/{slug}` preferred, else `Reference/Values/{slug}`
/// - If still unresolved: `/Web/CSS/{slug}` (typically a 404 link)
///
/// Values pages with conventional `_value` / `_function` suffixes are also
/// indexed under their suffix-less names, so a stripped slug like `color`
/// resolves to `Reference/Values/color_value`.
///
/// # Special handling
/// - Functions automatically get `()` appended to display text if not present
/// - Data types get wrapped in `<>` brackets in display text if not present
/// - Handles HTML entity encoding (`&lt;` and `&gt;`)
/// - Function inputs (`...()`) prefer a `{slug}_function` page when one exists,
///   so `fit-content()` and `:host()` resolve to the function page rather than
///   the like-named keyword/selector page
#[rari_f(register = "crate::Templ")]
pub fn cssxref(
    name: String,
    display: Option<String>,
    anchor: Option<String>,
) -> Result<String, DocError> {
    let display = display.as_deref().filter(|s| !s.is_empty());
    cssxref_internal(&name, display, anchor.as_deref(), env.locale)
}

pub fn cssxref_internal(
    name: &str,
    display_name: Option<&str>,
    anchor: Option<&str>,
    locale: Locale,
) -> Result<String, DocError> {
    // Authors sometimes embed a fragment in `name` (e.g.
    // `font-variant-alternates#stylistic` or `<color>#syntax`) instead of
    // passing the explicit `anchor` argument. Split it off up front so
    // special-case mapping, prefix classification, and slug normalization
    // all operate on the bare name; re-append it when building the URL.
    let (name, embedded_anchor) = match name.split_once('#') {
        Some((n, frag)) => (n, Some(frag)),
        None => (name, None),
    };
    if embedded_anchor.is_some() && anchor.is_some() {
        let ic = get_issue_counter();
        tracing::warn!(
            source = "templ-invalid-arg",
            ic = ic,
            arg = "anchor",
            "cssxref: `anchor` argument ignored because `name` already contains a fragment"
        );
    }

    let maybe_display_name = &display_name
        .or_else(|| name.rsplit_once('/').map(|(_, s)| s))
        .unwrap_or(name);
    let decoded_maybe_display_name = html_escape::decode_html_entities(maybe_display_name);
    let encoded_maybe_display_name = html_escape::encode_text(decoded_maybe_display_name.as_ref());

    // Strip type brackets and function parens to get the raw slug.
    let mut slug = name
        .strip_prefix("&lt;")
        .unwrap_or(name.strip_prefix('<').unwrap_or(name));
    slug = slug
        .strip_suffix("&gt;")
        .unwrap_or(slug.strip_suffix('>').unwrap_or(slug));
    let is_function = slug.ends_with("()");
    slug = slug.strip_suffix("()").unwrap_or(slug);

    let base_url = format!("/{}/docs/Web/CSS/", locale.as_url_str());
    // Resolve the (normalized) slug to a canonical Web/CSS sub-path via the
    // CSS feature index. The macro's input syntax (`<>`, `()`, `:`, `@`)
    // narrows the category we look up under. For function inputs we look up
    // `{slug}_function` first so we land directly on the function page when
    // both `{slug}` and `{slug}_function` exist (e.g. `fit-content()`,
    // `:host()`); bare slugs are ambiguous between a property and a value,
    // so try `Properties/` first then `Values/` — matching the legacy
    // behavior. If the index returns nothing, fall back to a bare `{slug}`
    // link under `Web/CSS/` (likely a 404).
    let url_path = if name.starts_with("&lt;") || name.starts_with('<') {
        resolve_css_feature("Values", slug)
    } else if name.starts_with(':') {
        is_function
            .then(|| resolve_css_feature("Selectors", &format!("{slug}_function")))
            .flatten()
            .or_else(|| resolve_css_feature("Selectors", slug))
    } else if name.starts_with('@') {
        resolve_css_feature("At-rules", slug)
    } else if is_function {
        resolve_css_feature("Values", &format!("{slug}_function"))
            .or_else(|| resolve_css_feature("Values", slug))
    } else {
        resolve_css_feature("Properties", slug).or_else(|| resolve_css_feature("Values", slug))
    }
    .map(str::to_string)
    .unwrap_or_else(|| slug.to_string());

    let fragment = embedded_anchor
        .map(|a| format!("#{a}"))
        .or_else(|| anchor.map(str::to_string))
        .unwrap_or_default();
    let url = format!("{}{}{}", &base_url, &url_path, &fragment);

    let display_name = if display_name.is_some() {
        encoded_maybe_display_name.to_string()
    } else if let Ok(doc) = RariApi::get_page_nowarn(&url) {
        match doc.page_type() {
            PageType::CssFunction if !encoded_maybe_display_name.ends_with("()") => {
                format!("{encoded_maybe_display_name}()")
            }
            PageType::CssType
                if !(encoded_maybe_display_name.starts_with("&lt;")
                    && encoded_maybe_display_name.ends_with("&gt;")) =>
            {
                format!("&lt;{encoded_maybe_display_name}&gt;")
            }
            _ => encoded_maybe_display_name.to_string(),
        }
    } else {
        encoded_maybe_display_name.to_string()
    };
    RariApi::link(&url, Some(locale), Some(&display_name), true, None, false)
}
