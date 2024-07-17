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
    let selector = Selector::parse("*[data-update-id], h2:not([id]), h3:not([id])").unwrap();
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
                let id = anchorize(&text);
                (el.id(), id)
            })
            .collect::<Vec<_>>();
    for (el_id, id) in subs {
        add_attribute(html, el_id, "id", &id);
        remove_attribute(html, el_id, "data-update-id");
    }
    Ok(())
}
