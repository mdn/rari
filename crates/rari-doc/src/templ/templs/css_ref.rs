use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use itertools::Itertools;
use rari_templ_func::rari_f;
use rari_utils::concat_strs;
use regex::Regex;
use serde_json::Value;

use crate::error::DocError;
use crate::helpers::css_info::mdn_data_files;
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

#[rari_f]
pub fn css_ref() -> Result<String, DocError> {
    let data = mdn_data_files();

    let mut index = BTreeMap::<char, HashMap<Cow<'static, str>, Cow<'static, str>>>::new();

    for (prop_name, prop) in &data.css_properties {
        if !matches!(prop["status"].as_str(), Some("experimental" | "standard")) {
            continue;
        }
        let initial = initial_letter(prop_name);
        let entry = index.entry(initial).or_default();
        let (url, label) = adjust_output(Cow::Borrowed(prop_name), Cow::Borrowed(prop_name));
        entry.entry(url).or_insert(label);
    }

    for (type_name, typ) in &data.css_types {
        if !matches!(typ["status"].as_str(), Some("experimental" | "standard")) {
            continue;
        }
        let initial = initial_letter(type_name);
        let entry = index.entry(initial).or_default();

        let url_path: Cow<'static, str> = match type_name.as_str() {
            "color" | "flex" | "position" => Cow::Owned(concat_strs!(type_name.as_str(), "_value")),
            _ => Cow::Borrowed(type_name),
        };
        let label = if type_name.starts_with('<') {
            Cow::Borrowed(type_name.as_str())
        } else {
            Cow::Owned(concat_strs!("<", type_name, ">"))
        };
        let (url, label) = adjust_output(url_path, label);
        entry.entry(url).or_insert(label);
    }

    for (syntax_name, syntax) in &data.css_syntaxes {
        if let Some(url_path) = syntax_name.strip_suffix("()") {
            let initial = initial_letter(url_path);
            let entry = index.entry(initial).or_default();

            let (url, label) = adjust_output(Cow::Borrowed(url_path), Cow::Borrowed(syntax_name));
            entry.entry(url).or_insert(label);
        }
        if let Value::String(syntax) = &syntax["syntax"] {
            for syntax_item in items_from_syntax(syntax) {
                if data.css_syntaxes.contains_key(syntax_item.as_ref()) {
                    continue;
                }

                let initial = initial_letter(&syntax_item);
                let entry = index.entry(initial).or_default();

                let url_path = syntax_item
                    .strip_suffix("()")
                    .map(|s| Cow::Owned(s.to_string()))
                    .unwrap_or(syntax_item.clone());
                let label = syntax_item.clone();
                let (url, label) = adjust_output(url_path, label);
                entry.entry(url).or_insert(label);
            }
        }
    }

    for (at_rule_name, rule) in &data.css_at_rules {
        if !matches!(rule["status"].as_str(), Some("experimental" | "standard")) {
            continue;
        }

        let initial = initial_letter(at_rule_name);
        let entry = index.entry(initial).or_default();
        let (url, label) = adjust_output(Cow::Borrowed(at_rule_name), Cow::Borrowed(at_rule_name));
        entry.entry(url).or_insert(label);

        if let Value::String(syntax) = &rule["syntax"] {
            for syntax_item in items_from_syntax(syntax) {
                if &syntax_item == at_rule_name {
                    continue;
                }
                let initial = initial_letter(&syntax_item);
                let entry = index.entry(initial).or_default();

                let url_path = Cow::Owned(concat_strs!(
                    at_rule_name.as_str(),
                    "/",
                    syntax_item.as_ref()
                ));
                let label = Cow::Owned(concat_strs!(syntax_item.as_ref(), " (", at_rule_name, ")"));
                let (url, label) = adjust_output(url_path, label);
                entry.entry(url).or_insert(label);
            }
        }

        if let Value::Object(descriptors) = &rule["descriptors"] {
            for (d_name, descriptor) in descriptors {
                if !matches!(
                    descriptor["status"].as_str(),
                    Some("experimental" | "standard")
                ) {
                    continue;
                }

                let initial = initial_letter(d_name);
                let entry = index.entry(initial).or_default();

                let url_path = Cow::Owned(concat_strs!(at_rule_name.as_str(), "/", d_name));
                let label = Cow::Owned(concat_strs!(d_name, " (", at_rule_name, ")"));
                let (url, label) = adjust_output(url_path, label);
                entry.entry(url).or_insert(label);

                if let Value::String(syntax) = &descriptor["syntax"] {
                    for syntax_item in items_from_syntax(syntax) {
                        let initial = initial_letter(&syntax_item);
                        let entry = index.entry(initial).or_default();

                        let url_path = Cow::Owned(concat_strs!(
                            at_rule_name.as_str(),
                            "/",
                            d_name,
                            "#",
                            syntax_item.as_ref()
                        ));
                        let (url, label) = adjust_output(url_path, syntax_item);
                        entry.entry(url).or_insert(label);
                    }
                }
            }
        }
    }

    for (selector_name, selector) in &data.css_seclectors {
        if !matches!(
            selector["status"].as_str(),
            Some("experimental" | "standard")
        ) {
            continue;
        }

        // Exclude basic selectors
        if selector_name.contains(' ') {
            continue;
        }

        let initial = initial_letter(selector_name);
        let entry = index.entry(initial).or_default();
        let (url, label) =
            adjust_output(Cow::Borrowed(selector_name), Cow::Borrowed(selector_name));
        entry.entry(url).or_insert(label);
    }

    for (unit_name, unit) in &data.css_units {
        if !matches!(unit["status"].as_str(), Some("experimental" | "standard")) {
            continue;
        }
        let initial = initial_letter(unit_name);
        let entry = index.entry(initial).or_default();
        let unit_name = match unit["groups"][1].as_str().unwrap_or("---") {
            "CSS Lengths" => Cow::Owned(concat_strs!("length#", unit_name)),
            "CSS Angles" => Cow::Owned(concat_strs!("angle#", unit_name)),
            "CSS Flexible Lengths" => Cow::Owned(concat_strs!("flex_value#", unit_name)),
            "CSS Frequencies" => Cow::Owned(concat_strs!("frequency#", unit_name)),
            "CSS Times" => Cow::Owned(concat_strs!("time#", unit_name)),
            "CSS Resolutions" => Cow::Owned(concat_strs!("resolution#", unit_name)),
            _ => Cow::Borrowed(unit_name.as_str()),
        };
        let (url, label) = adjust_output(unit_name.clone(), unit_name);
        entry.entry(url).or_insert(label);
    }

    for (letter, url_path, label) in [
        ('C', "counter", "<counter>"),
        ('U', "url#The_url()_functional_notation", "url()"),
        ('I', "inherit", "inherit"),
        ('I', "initial", "initial"),
        ('R', "revert", "revert"),
        ('U', "unset", "unset"),
        ('V', "var", "var()"),
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
        // Font-feature-values
        "@annotation" | "@character-variant" | "@historical-forms" | "@ornaments" | "@styleset"
        | "@stylistic" | "@swash" => {
            Cow::Owned(concat_strs!("@font-feature-values#", label.as_ref()))
        }

        // Font-variant-alternates
        "annotation()"
        | "character-variant()"
        | "ornaments()"
        | "styleset()"
        | "stylistic()"
        | "swash()" => Cow::Owned(concat_strs!("@font-variant-alternates#", label.as_ref())),

        // Image
        "image()" => Cow::Borrowed("image#The_image()_functional_notation"),
        "image-set()" | "paint()" => {
            Cow::Owned(concat_strs!("image/", label.trim_end_matches("()")))
        }

        // Filter
        "blur()" | "brightness()" | "contrast()" | "drop-shadow()" | "grayscale()"
        | "hue-rotate()" | "invert()" | "opacity()" | "saturate()" | "sepia()" => Cow::Owned(
            concat_strs!("filter-function/", label.trim_end_matches("()")),
        ),

        // Transforms
        "matrix()" | "matrix3d()" | "perspective()" | "rotate()" | "rotate3d()" | "rotateX()"
        | "rotateY()" | "rotateZ()" | "scale()" | "scale3d()" | "scaleX()" | "scaleY()"
        | "scaleZ()" | "skew()" | "skewX()" | "skewY()" | "translate()" | "translate3d()"
        | "translateX()" | "translateY()" | "translateZ()" => Cow::Owned(concat_strs!(
            "transform-function/",
            label.trim_end_matches("()")
        )),

        // Colors
        "rgba()" => return (Cow::Borrowed("color_value/rgb"), label),
        "hsla()" => return (Cow::Borrowed("color_value/hsl"), label),
        "rgb()" | "hsl()" | "hwb()" | "lab()" | "lch()" | "light-dark()" | "oklab()"
        | "oklch()" => Cow::Owned(concat_strs!("color_value/", label.trim_end_matches("()"))),

        // Gradients
        "conic-gradient()"
        | "linear-gradient()"
        | "radial-gradient()"
        | "repeating-conic-gradient()"
        | "repeating-linear-gradient()"
        | "repeating-radial-gradient()" => {
            Cow::Owned(concat_strs!("gradient/", label.trim_end_matches("()")))
        }

        // Shapes
        "inset()" | "polygon()" | "circle()" | "ellipse()" => {
            Cow::Owned(concat_strs!("basic-shape#", label.as_ref()))
        }
        "rect()" => Cow::Owned(concat_strs!("shape#", label.as_ref())),

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
        | "@right-bottom" => Cow::Borrowed("@page#page-margin-box-type"),

        // Easing functions
        "cubic-bezier()" => Cow::Borrowed("easing-function#cubic_bézier_easing_function"),
        "linear()" => Cow::Borrowed("easing-function#linear_easing_function"),
        "steps()" => Cow::Borrowed("easing-function#steps_easing_function"),

        // Alternate name
        "word-wrap" => Cow::Borrowed("overflow-wrap"),
        "line-clamp" => Cow::Borrowed("-webkit-line-clamp"),

        // Misc
        "view()" => Cow::Borrowed("animation-timeline/view"),
        ":host()" => Cow::Borrowed(":host_function"),
        "format()" => Cow::Borrowed("@font-face/src#format()"),
        "url()" => Cow::Borrowed("url#The_url()_functional_notation"),

        _ => url,
    };
    (url, label)
}
