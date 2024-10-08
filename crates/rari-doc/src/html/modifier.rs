use std::borrow::Cow;
use std::collections::HashSet;

use ego_tree::NodeId;
use html5ever::{namespace_url, ns, QualName};
use rari_md::anchor::anchorize;
use scraper::{ElementRef, Html, Node, Selector};

use crate::error::DocError;

pub fn add_attribute(html: &mut Html, node_id: NodeId, key: &str, value: &str) {
    if let Some(mut details) = html.tree.get_mut(node_id) {
        if let Node::Element(ref mut el) = details.value() {
            el.attrs.insert(
                QualName {
                    prefix: None,
                    ns: ns!(),
                    local: key.into(),
                },
                value.into(),
            );
        }
    }
}

pub fn remove_attribute(html: &mut Html, node_id: NodeId, key: &str) {
    if let Some(mut details) = html.tree.get_mut(node_id) {
        if let Node::Element(ref mut el) = details.value() {
            el.attrs.swap_remove(&QualName {
                prefix: None,
                ns: ns!(),
                local: key.into(),
            });
        }
    }
}

pub fn add_missing_ids(html: &mut Html) -> Result<(), DocError> {
    let selector = Selector::parse("*[id]").unwrap();
    let mut ids = html
        .select(&selector)
        .filter_map(|el| el.attr("id"))
        .map(Cow::Borrowed)
        .collect::<HashSet<_>>();

    let selector =
        Selector::parse("*[data-update-id], h2:not([id]), h3:not([id]), dt:not([id])").unwrap();
    let subs =
        html.select(&selector)
            .map(|el| {
                let text = if let Some(text) = el.first_child().and_then(|child| {
                    ElementRef::wrap(child).map(|el| el.text().collect::<String>())
                }) {
                    text
                } else {
                    el.text().collect::<String>()
                };
                let mut id = anchorize(&text);
                if ids.contains(id.as_ref()) {
                    let (prefix, mut count) = if let Some((prefix, counter)) = id.rsplit_once('_') {
                        if counter.chars().all(|c| c.is_ascii_digit()) {
                            let count = counter.parse::<i64>().unwrap_or_default() + 1;
                            (prefix, count)
                        } else {
                            (id.as_ref(), 2)
                        }
                    } else {
                        (id.as_ref(), 2)
                    };
                    let mut new_id = format!("{prefix}_{count}");
                    while ids.contains(new_id.as_str()) && count < 666 {
                        count += 1;
                        new_id = format!("{prefix}_{count}");
                    }
                    id = Cow::Owned(new_id);
                }
                let id_ = id.to_string();
                ids.insert(Cow::Owned(id.into_owned()));
                (el.id(), id_)
            })
            .collect::<Vec<_>>();
    for (el_id, id) in subs {
        add_attribute(html, el_id, "id", &id);
        remove_attribute(html, el_id, "data-update-id");
    }
    Ok(())
}

pub fn remove_empty_p(html: &mut Html) -> Result<(), DocError> {
    let selector = Selector::parse("p:empty").unwrap();
    let dels = html.select(&selector).map(|el| el.id()).collect::<Vec<_>>();

    for id in dels {
        html.tree.get_mut(id).unwrap().detach();
    }

    Ok(())
}
