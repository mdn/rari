use rari_templ_func::rari_f;
use rari_types::AnyArg;

use crate::error::DocError;
use crate::issues::get_issue_counter;
use crate::templ::api::RariApi;
use crate::templ::js_ref_index::resolve_js_ref;

/// Creates a link to a JavaScript reference page on MDN.
///
/// This macro generates links to JavaScript language features including objects,
/// methods, properties, statements, operators, and other JavaScript reference
/// documentation. It resolves the API name against an index of all
/// `Web/JavaScript/Reference/*` pages (see the "Name resolution" section below).
///
/// # Arguments
/// * `api_name` - The JavaScript feature name (object, method, property, etc.)
/// * `display` - Optional custom display text for the link
/// * `anchor` - Optional anchor/fragment to append to the URL
/// * `no_code` - Optional flag to disable code formatting (default: false)
///
/// # Examples
/// * `{{JSxRef("Array")}}` -> links to Array global object
/// * `{{JSxRef("Array.prototype.map")}}` -> links to Array map method
/// * `{{JSxRef("Promise", "Promises")}}` -> custom display text
/// * `{{JSxRef("if...else")}}` -> links to if...else statement
/// * `{{JSxRef("typeof", "", "", true)}}` -> disables code formatting
///
/// # Special handling
/// - Removes `()` from method names for URL generation
/// - Converts `.prototype.` notation to `/` for URL paths
/// - Falls back to URI component decoding if no page found
/// - Formats links with `<code>` tags unless `no_code` is true
///
/// # Name resolution
/// The normalized `api_name` is resolved against an index of all
/// `Web/JavaScript/Reference/*` pages (see [`crate::templ::js_ref_index`]).
/// Authors can use:
/// - A full sub-path: `{{JSxRef("Statements/for...of")}}`
/// - A bare global-object name or dotted member:
///   `{{JSxRef("Array")}}`, `{{JSxRef("Array.prototype.map")}}`
/// - **For namespace-class members only** (`Intl`, `Temporal`), a path with
///   the namespace omitted: `{{JSxRef("Collator")}}` resolves to
///   `Intl/Collator`, `{{JSxRef("Collator/compare")}}` to
///   `Intl/Collator/compare`.
///
/// A fragment may be embedded directly in `api_name`
/// (e.g. `{{JSxRef("Array.prototype.map#examples")}}`) instead of passing the
/// separate `anchor` argument; it is split off before resolution and
/// re-appended to the URL. If both are given, the embedded fragment wins and
/// the `anchor` argument is ignored (with a `templ-invalid-arg` flaw).
#[rari_f(register = "crate::Templ")]
pub fn jsxref(
    api_name: String,
    display: Option<String>,
    anchor: Option<String>,
    no_code: Option<AnyArg>,
) -> Result<String, DocError> {
    let parts = jsxref_parts(&api_name, display.as_deref(), anchor.as_deref());
    if parts.anchor_conflict {
        let ic = get_issue_counter();
        tracing::warn!(
            source = "templ-invalid-arg",
            ic = ic,
            arg = "anchor",
            "jsxref: `anchor` argument ignored because `api_name` already contains a fragment"
        );
    }

    let base = format!("/{}/docs/Web/JavaScript/Reference/", env.locale);
    let mut url = if let Some(resolved) = resolve_js_ref(&parts.normalized) {
        format!("{base}{resolved}")
    } else {
        format!("{base}{}", RariApi::decode_uri_component(parts.bare))
    };

    if let Some(anchor) = parts.anchor {
        push_anchor(&mut url, anchor);
    }

    let code = !no_code.map(|nc| nc.as_bool()).unwrap_or_default();
    RariApi::link(
        &url,
        Some(env.locale),
        Some(parts.display),
        code,
        None,
        false,
    )
}

/// The resolution decisions for a `jsxref` invocation, factored out of
/// [`jsxref`] so the fragment/anchor/normalization logic is unit-testable
/// without an `env` or the page index. The index lookup and the
/// `RariApi::link` call stay in [`jsxref`].
struct JsxrefParts<'a> {
    /// Fragment-stripped, un-normalized name, used for the decode fallback
    /// when the index lookup misses.
    bare: &'a str,
    /// `bare` with `()` stripped, `.prototype.` → `.`, and `.` → `/` (unless a
    /// `/` is already present), ready for the index lookup.
    normalized: String,
    /// Display text: the explicit non-empty `display`, else `bare`.
    display: &'a str,
    /// Fragment to append (without a leading `#`): the embedded fragment if
    /// present, else the explicit non-empty `anchor`.
    anchor: Option<&'a str>,
    /// Both an embedded fragment and an explicit `anchor` were supplied; the
    /// embedded one wins and the caller emits a `templ-invalid-arg` flaw.
    anchor_conflict: bool,
}

/// Authors sometimes embed a fragment in `api_name` (e.g. `Array/map#examples`
/// or `Array.prototype.map#examples`) instead of passing the explicit `anchor`
/// argument. Split it off up front so normalization and the index lookup
/// operate on the bare name; the embedded fragment wins over `anchor`.
fn jsxref_parts<'a>(
    api_name: &'a str,
    display: Option<&'a str>,
    anchor: Option<&'a str>,
) -> JsxrefParts<'a> {
    let (bare, embedded_anchor) = match api_name.split_once('#') {
        Some((n, frag)) => (n, Some(frag)),
        None => (api_name, None),
    };
    let anchor = anchor.filter(|s| !s.is_empty());
    let anchor_conflict = embedded_anchor.is_some() && anchor.is_some();

    let display = display.filter(|s| !s.is_empty()).unwrap_or(bare);

    let normalized = bare.replace("()", "").replace(".prototype.", ".");
    let normalized = if !normalized.contains('/') && normalized.contains('.') {
        normalized.replace('.', "/")
    } else {
        normalized
    };

    JsxrefParts {
        bare,
        normalized,
        display,
        anchor: embedded_anchor.or(anchor),
        anchor_conflict,
    }
}

/// Append `anchor` (with or without a leading `#`) to `url` as a fragment.
fn push_anchor(url: &mut String, anchor: &str) {
    if !anchor.starts_with('#') {
        url.push('#');
    }
    url.push_str(anchor);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsxref_parts_cases() {
        struct Case {
            name: &'static str,
            api_name: &'static str,
            display: Option<&'static str>,
            anchor: Option<&'static str>,
            bare: &'static str,
            normalized: &'static str,
            display_out: &'static str,
            anchor_out: Option<&'static str>,
            conflict: bool,
        }
        let cases = vec![
            Case {
                name: "plain name",
                api_name: "Array",
                display: None,
                anchor: None,
                bare: "Array",
                normalized: "Array",
                display_out: "Array",
                anchor_out: None,
                conflict: false,
            },
            Case {
                name: "strips parens",
                api_name: "Array.prototype.map()",
                display: None,
                anchor: None,
                bare: "Array.prototype.map()",
                normalized: "Array/map",
                display_out: "Array.prototype.map()",
                anchor_out: None,
                conflict: false,
            },
            Case {
                name: "dotted member to slash",
                api_name: "Array.prototype.map",
                display: None,
                anchor: None,
                bare: "Array.prototype.map",
                normalized: "Array/map",
                display_out: "Array.prototype.map",
                anchor_out: None,
                conflict: false,
            },
            Case {
                name: "sub-path kept as-is",
                api_name: "Statements/for...of",
                display: None,
                anchor: None,
                bare: "Statements/for...of",
                normalized: "Statements/for...of",
                display_out: "Statements/for...of",
                anchor_out: None,
                conflict: false,
            },
            Case {
                name: "embedded fragment split off",
                api_name: "Array.prototype.map#examples",
                display: None,
                anchor: None,
                bare: "Array.prototype.map",
                normalized: "Array/map",
                display_out: "Array.prototype.map",
                anchor_out: Some("examples"),
                conflict: false,
            },
            Case {
                name: "explicit anchor",
                api_name: "Array",
                display: None,
                anchor: Some("examples"),
                bare: "Array",
                normalized: "Array",
                display_out: "Array",
                anchor_out: Some("examples"),
                conflict: false,
            },
            Case {
                name: "empty anchor filtered out",
                api_name: "Array",
                display: None,
                anchor: Some(""),
                bare: "Array",
                normalized: "Array",
                display_out: "Array",
                anchor_out: None,
                conflict: false,
            },
            Case {
                name: "embedded fragment wins over explicit anchor",
                api_name: "Array#embedded",
                display: None,
                anchor: Some("explicit"),
                bare: "Array",
                normalized: "Array",
                display_out: "Array",
                anchor_out: Some("embedded"),
                conflict: true,
            },
            Case {
                name: "explicit display wins",
                api_name: "Array",
                display: Some("the array"),
                anchor: None,
                bare: "Array",
                normalized: "Array",
                display_out: "the array",
                anchor_out: None,
                conflict: false,
            },
            Case {
                name: "empty display falls back to bare name",
                api_name: "Array.prototype.map#x",
                display: Some(""),
                anchor: None,
                bare: "Array.prototype.map",
                normalized: "Array/map",
                display_out: "Array.prototype.map",
                anchor_out: Some("x"),
                conflict: false,
            },
        ];
        for c in cases {
            let parts = jsxref_parts(c.api_name, c.display, c.anchor);
            assert_eq!(parts.bare, c.bare, "bare [{}]", c.name);
            assert_eq!(parts.normalized, c.normalized, "normalized [{}]", c.name);
            assert_eq!(parts.display, c.display_out, "display [{}]", c.name);
            assert_eq!(parts.anchor, c.anchor_out, "anchor [{}]", c.name);
            assert_eq!(parts.anchor_conflict, c.conflict, "conflict [{}]", c.name);
        }
    }

    #[test]
    fn push_anchor_adds_missing_hash() {
        let mut url = String::from("/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array");
        push_anchor(&mut url, "examples");
        assert!(url.ends_with("/Array#examples"));
    }

    #[test]
    fn push_anchor_keeps_existing_hash() {
        let mut url = String::from("/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array");
        push_anchor(&mut url, "#examples");
        assert!(url.ends_with("/Array#examples"));
    }
}
