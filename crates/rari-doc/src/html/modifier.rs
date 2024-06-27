use ego_tree::NodeId;
use html5ever::{namespace_url, ns, QualName};
use rari_md::anchor::anchorize;
use scraper::{Html, Node, Selector};

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

pub fn add_missing_ids(html: &mut Html) -> Result<(), DocError> {
    let a_selector = Selector::parse("*[id=---update-id]").unwrap();
    let subs = html
        .select(&a_selector)
        .map(|el| {
            let text = if let Some(text) = el
                .first_child()
                .and_then(|child| child.value().as_text().map(|t| t.to_string()))
            {
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
    }
    Ok(())
}
