use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use itertools::Itertools;
use rari_templ_func::rari_f;
use rari_utils::concat_strs;
use regex::Regex;

use crate::error::DocError;
use crate::helpers::css_info::css_ref_data;
use crate::templ::api::RariApi;

static SYNTAX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"([\w-]+)\([\w\-<>,#+*?\[\]|\/ ]*\)|(@[\w-]+)"#).unwrap());

fn items_from_syntax(syntax: &str) -> Vec<Cow<'_, str>> {
    SYNTAX_RE
        .captures_iter(syntax)
        .filter_map(|cap| cap.iter().skip(1).find(|c| c.is_some()))
        .flatten()
        .map(|i| {
            let name = i.as_str();
            if name.starts_with('@') {
                Cow::Borrowed(name)
            } else {
                Cow::Owned(concat_strs!(name, "()"))
            }
        })
        .collect()
}

static SKIPPED_ENTRIES: &[&str] = &[
    "(-token",
    ")-token",
    "[-token",
    "]-token",
    "{-token",
    "}-token",
    "CDC-token",
    "CDO-token",
    " ",
    "&",
    "+",
    "",
    "||",
    "~",
    ":after",
    ":before",
    ":first-letter",
    ":first-line",
];

#[rari_f(register = "crate::Templ")]
pub fn css_ref() -> Result<String, DocError> {
    let mut index = BTreeMap::<char, HashMap<Cow<'static, str>, Cow<'static, str>>>::new();

    let prop_data = css_ref_data().properties.get("__global_scope__").unwrap();
    for (prop_name, prop) in prop_data {
        // Skip properties with legacy aliases, usually `-webkit-*` props
        if prop.legacy_alias_of.is_some() {
            continue;
        }
        let initial = initial_letter(prop_name);
        let entry = index.entry(initial).or_default();
        let (url, label) = adjust_output(
            Cow::Owned(concat_strs!("Reference/Properties/", prop_name)),
            Cow::Borrowed(prop_name),
        );
        println!("============ CSSREF prop entry {url} {label}");
        entry.entry(url).or_insert(label);
    }

    let type_data = css_ref_data().types.get("__global_scope__").unwrap();
    for type_name in type_data.keys() {
        let initial = initial_letter(type_name);
        let entry = index.entry(initial).or_default();

        if SKIPPED_ENTRIES.contains(&type_name.as_str()) {
            continue;
        }
        let label = if type_name.starts_with('<') {
            Cow::Borrowed(type_name.as_str())
        } else {
            Cow::Owned(concat_strs!("<", type_name, ">"))
        };
        let (url, label) = adjust_output(
            Cow::Owned(concat_strs!("Reference/Values/", type_name)),
            label,
        );
        println!("============ CSSREF type entry {url} {label}");
        entry.entry(url).or_insert(label);
    }

    let atrule_data = css_ref_data().atrules.get("__global_scope__").unwrap();
    for (at_rule_name, rule) in atrule_data {
        let initial = initial_letter(at_rule_name);
        let entry = index.entry(initial).or_default();
        let (url, label) = adjust_output(
            Cow::Owned(concat_strs!("Reference/At-rules/", at_rule_name)),
            Cow::Borrowed(at_rule_name),
        );
        entry.entry(url).or_insert(label);

        // main at rule
        if let Some(syntax) = &rule.syntax {
            for syntax_item in items_from_syntax(syntax) {
                if &syntax_item == at_rule_name {
                    println!(
                        "============ CSSREF Skipping self-reference {syntax_item} | {at_rule_name}"
                    );
                    continue;
                }
                let initial = initial_letter(&syntax_item);
                let entry = index.entry(initial).or_default();

                let url_path = Cow::Owned(concat_strs!(
                    "Reference/At-rules/",
                    at_rule_name.as_str(),
                    "/",
                    syntax_item.as_ref()
                ));
                let label = Cow::Owned(concat_strs!(syntax_item.as_ref(), " (", at_rule_name, ")"));
                let (url, label) = adjust_output(url_path, label);
                println!("============ CSSREF atrule  entry {url} {label}");

                entry.entry(url).or_insert(label);
            }
        }

        // at rule descriptors
        for (d_name, descriptor) in &rule.descriptors {
            let initial = initial_letter(d_name);
            let entry = index.entry(initial).or_default();

            let url_path = Cow::Owned(concat_strs!(
                "Reference/At-rules/",
                at_rule_name,
                "/",
                &d_name
            ));
            let label = Cow::Owned(concat_strs!(&d_name, " (", at_rule_name, ")"));
            let (url, label) = adjust_output(url_path, label);
            entry.entry(url).or_insert(label);

            if let Some(syntax) = &descriptor.syntax {
                for syntax_item in items_from_syntax(syntax) {
                    let initial = initial_letter(&syntax_item);
                    let entry = index.entry(initial).or_default();

                    if SKIPPED_ENTRIES.contains(&d_name.as_str()) {
                        continue;
                    }
                    let url_path = Cow::Owned(concat_strs!(
                        "Reference/At-rules/",
                        at_rule_name,
                        "/",
                        &d_name,
                        "#",
                        syntax_item.as_ref()
                    ));
                    let (url, label) = adjust_output(url_path, syntax_item);
                    println!(
                        "============ CSSREF atrule descriptor entry {url} {label} | {d_name}"
                    );
                    entry.entry(url).or_insert(label);
                }
            }
        }
    }

    let selector_data = css_ref_data().selectors.get("__global_scope__").unwrap();
    for selector_name in selector_data.keys() {
        // Exclude basic selectors
        if selector_name.contains(' ') {
            continue;
        }
        if SKIPPED_ENTRIES.contains(&selector_name.as_str()) {
            continue;
        }

        let initial = initial_letter(selector_name);
        let entry = index.entry(initial).or_default();
        let (url, label) = adjust_output(
            Cow::Owned(concat_strs!("Reference/Selectors/", selector_name)),
            Cow::Borrowed(selector_name),
        );
        println!("============ CSSREF selector {url} {label}");
        entry.entry(url).or_insert(label);
    }

    let function_data = css_ref_data().functions.get("__global_scope__").unwrap();
    for function_name in function_data.keys() {
        if SKIPPED_ENTRIES.contains(&function_name.as_str()) {
            continue;
        }

        let initial = initial_letter(function_name);
        let entry = index.entry(initial).or_default();
        let (url, label) = adjust_output(
            Cow::Owned(concat_strs!("Reference/Values/", function_name)),
            Cow::Borrowed(function_name),
        );
        println!("============ CSSREF function {url} {label}");
        entry.entry(url).or_insert(label);
    }

    for (letter, url_path, label) in [
        ('C', "counter", "<counter>"),
        ('U', "url#The_url()_functional_notation", "url()"),
        ('I', "inherit", "inherit"),
        ('I', "initial", "initial"),
        ('R', "revert", "revert"),
        ('U', "unset", "unset"),
        // ('V', "var", "var()"),
    ] {
        index
            .entry(letter)
            .or_default()
            .insert(Cow::Borrowed(url_path), Cow::Borrowed(label));
    }

    let mut out = String::new();
    out.push_str(r#"<div class="index">"#);
    for (letter, items) in index {
        out.push_str("<h3>");
        out.push(letter);
        out.push_str("</h3><ul>");
        for (url, label) in items
            .into_iter()
            .sorted_by(|(_, a), (_, b)| compare_items(a, b))
        {
            out.extend([
                "<li>",
                &RariApi::link(
                    &concat_strs!("/Web/CSS/", url.as_ref()),
                    Some(env.locale),
                    Some(&html_escape::encode_text(&label)),
                    true,
                    None,
                    false,
                )?,
                "</li>",
            ]);
        }
        out.push_str("</ul>");
    }
    out.push_str(r#"</div>"#);

    Ok(out)
}

fn compare_items(a: &str, b: &str) -> Ordering {
    let ord = a
        .trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '(' && c != ')' && c != '-')
        .cmp(
            b.trim_matches(|c: char| !c.is_ascii_alphabetic() && c != '(' && c != ')' && c != '-'),
        );
    if ord == Ordering::Equal {
        a.cmp(b)
    } else {
        ord
    }
}

fn initial_letter(s: &str) -> char {
    s.chars()
        .find(|&c| c.is_ascii_alphabetic() || c == '-')
        .unwrap_or('?')
        .to_ascii_uppercase()
}

fn adjust_output<'a>(url: Cow<'a, str>, label: Cow<'a, str>) -> (Cow<'a, str>, Cow<'a, str>) {
    let label = match label.as_ref() {
        // Add alternate name for pseudo-elements (one colon)
        "::after" | "::before" | "::first-letter" | "::first-line" => Cow::Owned(concat_strs!(
            label.as_ref(),
            " (",
            label.as_ref().strip_prefix(':').unwrap(),
            ")"
        )),
        _ => label,
    };

    let url = match label.as_ref() {
        // webkit stuff https://compat.spec.whatwg.org/#propdef--webkit-box-align
        "-webkit-box-align" => Cow::from("Reference/Properties/align-items"),
        "-webkit-box-flex" => Cow::from("Reference/Properties/flex-grow"),
        "-webkit-box-ordinal-group" => Cow::from("Reference/Properties/order"),
        "-webkit-box-orient" => Cow::from("Reference/Properties/flex-direction"),
        "-webkit-box-pack" => Cow::from("Reference/Properties/justify-content"),
        // https://compat.spec.whatwg.org/#at-ruledef--webkit-keyframes
        "@-webkit-keyframes" => Cow::from("Reference/At-rules/@keyframes"),
        // https://drafts.csswg.org/css-overflow-4/#propdef--webkit-line-clamp
        "-webkit-line-clamp" => Cow::from("Reference/Properties/line-clamp"),
        // https://drafts.csswg.org/css-ui-4/#propdef--webkit-user-select
        "-webkit-user-select" => Cow::from("Reference/Properties/user-select"),

        // Font-feature-values
        "@annotation" | "@character-variant" | "@historical-forms" | "@ornaments" | "@styleset"
        | "@stylistic" | "@swash" => Cow::Owned(concat_strs!(
            "Reference/At-rules/@font-feature-values#",
            label.as_ref()
        )),

        // Font-variant-alternates
        // "annotation()"
        // | "character-variant()"
        // | "ornaments()"
        // | "styleset()"
        // | "stylistic()"
        // | "swash()" => Cow::Owned(concat_strs!("@font-variant-alternates#", label.as_ref())),

        // Image
        "image()" => Cow::Borrowed("image#The_image()_functional_notation"),
        "image-set()" | "paint()" => {
            Cow::Owned(concat_strs!("image/", label.trim_end_matches("()")))
        }

        // Filter
        "blur()" | "brightness()" | "contrast()" | "drop-shadow()" | "grayscale()"
        | "hue-rotate()" | "invert()" | "opacity()" | "saturate()" | "sepia()" => {
            Cow::Owned(concat_strs!(
                "Reference/Values/filter-function/",
                label.trim_end_matches("()")
            ))
        }

        // Transforms
        "matrix()" | "matrix3d()" | "perspective()" | "rotate()" | "rotate3d()" | "rotateX()"
        | "rotateY()" | "rotateZ()" | "scale()" | "scale3d()" | "scaleX()" | "scaleY()"
        | "scaleZ()" | "skew()" | "skewX()" | "skewY()" | "translate()" | "translate3d()"
        | "translateX()" | "translateY()" | "translateZ()" => Cow::Owned(concat_strs!(
            "Reference/Values/transform-function/",
            label.trim_end_matches("()")
        )),

        // Colors
        "rgba()" => return (Cow::Borrowed("color_value/rgb"), label),
        "hsla()" => return (Cow::Borrowed("color_value/hsl"), label),
        "rgb()" | "hsl()" | "hwb()" | "lab()" | "lch()" | "light-dark()" | "oklab()"
        | "oklch()" => Cow::Owned(concat_strs!(
            "Reference/Values/color_value/",
            label.trim_end_matches("()")
        )),

        // Gradients
        "conic-gradient()"
        | "linear-gradient()"
        | "radial-gradient()"
        | "repeating-conic-gradient()"
        | "repeating-linear-gradient()"
        | "repeating-radial-gradient()" => Cow::Owned(concat_strs!(
            "Reference/Values/gradient/",
            label.trim_end_matches("()")
        )),

        // Shapes
        "inset()" | "polygon()" | "circle()" | "ellipse()" | "shape()" => Cow::Owned(concat_strs!(
            "Reference/Values/basic-shape#",
            label.as_ref()
        )),
        "rect()" => Cow::Owned(concat_strs!("Reference/Values/shape#", label.as_ref())),

        // @page
        "@top-left-corner"
        | "@top-left"
        | "@top-center"
        | "@top-right"
        | "@top-right-corner"
        | "@bottom-left-corner"
        | "@bottom-left"
        | "@bottom-center"
        | "@bottom-right"
        | "@bottom-right-corner"
        | "@left-top"
        | "@left-middle"
        | "@left-bottom"
        | "@right-top"
        | "@right-middle"
        | "@right-bottom" => Cow::Borrowed("Reference/At-rules/@page#page-margin-box-type"),

        // Easing functions
        "cubic-bezier()" => {
            Cow::Borrowed("Reference/Values/easing-function#cubic_bézier_easing_function")
        }
        "linear()" => Cow::Borrowed("Reference/Values/easing-function#linear_easing_function"),
        "steps()" => Cow::Borrowed("Reference/Values/easing-function#steps_easing_function"),

        // Alternate name
        "word-wrap" => Cow::Borrowed("Reference/Properties/overflow-wrap"),

        // Misc
        "view()" => Cow::Borrowed("Reference/Values/animation-timeline/view"),
        ":host()" => Cow::Borrowed("Reference/Selectors/:host_function"),
        "format()" => Cow::Borrowed("Reference/At-rules/@font-face/src#format()"),
        // "url()" => Cow::Borrowed("url#The_url()_functional_notation"),
        _ => url,
    };
    (url, label)
}
