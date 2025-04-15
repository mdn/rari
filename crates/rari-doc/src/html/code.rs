use std::borrow::Cow;
use std::collections::HashSet;

use ego_tree::NodeId;
use html5ever::{namespace_url, ns, QualName};
use rari_types::globals::settings;
use rari_utils::concat_strs;
use schemars::JsonSchema;
use scraper::{Element, ElementRef, Html, Node, Selector};
use serde::Serialize;

use super::ids::uniquify_id;
use super::modifier::insert_attribute;

#[derive(Debug, Default, Clone)]
pub struct CodeInternal {
    pub css: String,
    pub html: String,
    pub js: String,
    pub src: Option<String>,
    pub node_ids: Vec<NodeId>,
    pub id: String,
}

#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
pub struct Code {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub css: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub html: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub js: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub id: String,
}

impl From<CodeInternal> for Code {
    fn from(
        CodeInternal {
            css,
            html,
            js,
            src,
            id,
            ..
        }: CodeInternal,
    ) -> Self {
        Self {
            css,
            html,
            js,
            src,
            id,
        }
    }
}

pub fn code_blocks(html: &mut Html) -> Option<Vec<Code>> {
    let mut ids = HashSet::new();
    let mut update_data_live_id = Vec::new();
    let mut examples = vec![];
    let selector = Selector::parse("iframe[data-live-id]").ok()?;
    for iframe in html.select(&selector) {
        if let Some(id) = iframe.attr("data-live-id") {
            let src = iframe.attr("data-live-path").map(String::from);
            if let Some(mut code) = code_by_query(
                &html.root_element(),
                &concat_strs!("pre.live-sample___", id),
                None,
            ) {
                code.src = src;
                code.id = id.to_string();
                examples.push(code);
            } else {
                let id_ = uniquify_id(&mut ids, Cow::Borrowed(id));
                if id != id_ {
                    update_data_live_id.push((iframe.id(), id_.clone().to_string()));
                }
                let id = id_;

                let mut css_id = String::with_capacity(id.len() + 1);
                css_id.push('#');
                cssparser::serialize_identifier(&id, &mut css_id).unwrap();
                let selector = Selector::parse(&css_id);
                if let Ok(selector) = selector {
                    let mut next = html
                        .select(&selector)
                        .next()
                        .or_else(|| closest_heading(iframe));
                    {
                        while let Some(heading) = next {
                            if let Some(mut code) = gather_code(heading) {
                                code.src = src;
                                code.id = id.to_string();
                                examples.push(code);
                                break;
                            } else {
                                next = closest_parent_heading(heading)
                            }
                        }
                    }
                }
            }
        }
    }
    for (el_id, id) in update_data_live_id {
        insert_attribute(html, el_id, "data-live-id", &id);
    }

    let mut result = Vec::with_capacity(examples.len());
    for code in examples {
        for node_id in &code.node_ids {
            if let Some(mut node) = html.tree.get_mut(*node_id) {
                if let Node::Element(ref mut el) = node.value() {
                    let class = el.attr("class").unwrap_or_default();
                    let claas = concat_strs!(
                        class,
                        if class.is_empty() { "" } else { " " },
                        "live-sample---",
                        code.id.as_str()
                    );
                    el.attrs.insert(
                        QualName {
                            prefix: None,
                            ns: ns!(),
                            local: "class".into(),
                        },
                        claas.into(),
                    );
                }
            }
        }
        result.push(code.into())
    }

    if settings().json_live_samples {
        Some(result)
    } else {
        None
    }
}

fn gather_code(ref_element: ElementRef) -> Option<CodeInternal> {
    let h = ref_element.value().name();

    let block_mode = !is_heading(&ref_element);

    let mut code = None;
    if block_mode {
        code = code_by_query(&ref_element, "pre", code);
        return code;
    }

    let mut next = ref_element;
    while let Some(element) = next.next_sibling_element() {
        if is_heading(&element) && element.value().name() <= h {
            break;
        }
        code = code_by_query(&element, "pre", code);
        next = element;
    }

    code
}

fn code_by_query(
    root: &ElementRef,
    query: &str,
    code: Option<CodeInternal>,
) -> Option<CodeInternal> {
    let selector = Selector::parse(query).ok()?;
    root.select(&selector).fold(code, |acc, pre| {
        let mut acc = acc.unwrap_or_default();
        if pre.value().classes().any(|cls| match cls {
            "css" => {
                if !acc.css.is_empty() {
                    acc.css.push('\n');
                }
                acc.css.extend(pre.text());
                true
            }
            "js" => {
                if !acc.js.is_empty() {
                    acc.js.push('\n');
                }
                acc.js.extend(pre.text());
                true
            }
            "html" => {
                if !acc.html.is_empty() {
                    acc.html.push('\n');
                }
                acc.html.extend(pre.text());
                true
            }
            _ => false,
        }) {
            acc.node_ids.push(pre.id())
        };
        Some(acc)
    })
}

fn closest_parent_heading(heading: ElementRef) -> Option<ElementRef> {
    let h = heading.value().name();
    let mut next = heading;
    while let Some(element) = next
        .prev_sibling_element()
        .or_else(|| next.parent_element())
    {
        if is_major_heading(&element) && element.value().name() < h {
            return Some(element);
        } else {
            next = element;
        }
    }
    None
}

fn closest_heading(element: ElementRef) -> Option<ElementRef> {
    let mut next = element;
    while let Some(element) = next
        .prev_sibling_element()
        .or_else(|| next.parent_element())
    {
        if is_major_heading(&element) {
            return Some(element);
        } else {
            next = element;
        }
    }
    None
}

fn is_heading(element: &ElementRef) -> bool {
    matches!(
        element.value().name(),
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
    )
}

fn is_major_heading(element: &ElementRef) -> bool {
    matches!(element.value().name(), "h1" | "h2" | "h3")
}
