use std::borrow::Cow;
use std::collections::HashSet;

use ego_tree::NodeId;
use html5ever::{namespace_url, ns, Attribute, QualName};
use rari_md::anchor::anchorize;
use rari_utils::concat_strs;
use scraper::node::{self};
use scraper::{ElementRef, Html, Node, Selector};

use super::ids::uniquify_id;
use crate::error::DocError;
/// Inserts an attribute to a specified HTML node.
///
/// # Parameters
/// - `html`: A mutable reference to the HTML document structure.
/// - `node_id`: The ID of the node to which the attribute will be added.
/// - `key`: The name of the attribute to add.
/// - `value`: The value of the attribute to add.
///
/// If the node exists and is an element, this function adds or updates
/// the specified attribute in the node's attributes list.
pub fn insert_attribute(html: &mut Html, node_id: NodeId, key: &str, value: &str) {
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

/// Removes an attribute from a specified HTML node.
///
/// # Parameters
/// - `html`: A mutable reference to the HTML document structure.
/// - `node_id`: The ID of the node from which the attribute will be removed.
/// - `key`: The name of the attribute to remove.
///
/// If the node exists and is an element, this function removes the specified
/// attribute from the node's attributes list, if it exists.
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

/// Retrieves the `id` attribute of an HTML node if it exists, prefixed with `#`.
///
/// # Arguments
/// * `html` - A reference to the `Html` structure containing the node tree.
/// * `node_id` - The identifier of the node from which to retrieve the `id`.
///
/// # Returns
/// * `Option<String>` - Returns `Some(String)` containing the `id` prefixed with `#` if found, or `None` if the node
///   has no `id` attribute.
pub fn get_id(html: &Html, node_id: NodeId) -> Option<String> {
    if let Some(node) = html.tree.get(node_id) {
        if let Node::Element(node_el) = node.value() {
            if let Some(id) = node_el.attr("id") {
                return Some(concat_strs!("#", id));
            }
        }
    }
    None
}

/// Wraps the children of a specified node with a link element pointing to the node's own `id` attribute.
///
/// # Arguments
/// * `html` - A mutable reference to the `Html` structure to modify.
/// * `node_id` - The identifier of the node whose children will be wrapped with a link.
///
/// # Details
/// This function calls `get_id` to retrieve the `id` of the specified node and, if successful, wraps its children
/// with an anchor (`<a>`) link element using that `id` as the `href` attribute.
pub fn wrap_children_with_link_to_id(html: &mut Html, node_id: NodeId) {
    if let Some(id) = get_id(html, node_id) {
        wrap_children_with_link(html, node_id, id);
    }
}

/// Wraps the children of a specified node with a link element containing a specified `href`.
///
/// # Arguments
/// * `html` - A mutable reference to the `Html` structure to modify.
/// * `node_id` - The identifier of the node whose children will be wrapped with the link element.
/// * `href` - A `String` representing the `href` attribute for the new link element.
///
/// # Details
/// This function creates an anchor (`<a>`) element with the given `href`, then appends it as a child to the specified
/// node and reparents the node’s children to be inside the new link element.
pub fn wrap_children_with_link(html: &mut Html, node_id: NodeId, href: String) {
    let attribute = Attribute {
        name: QualName {
            prefix: None,
            ns: ns!(),
            local: "href".into(),
        },
        value: href.into(),
    };

    let a_node = Node::Element(node::Element::new(
        QualName {
            prefix: None,
            ns: ns!(),
            local: "a".into(),
        },
        vec![attribute],
    ));
    let mut a_node_ref = html.tree.orphan(a_node);
    a_node_ref.reparent_from_id_append(node_id);
    let a_node_id = a_node_ref.id();
    if let Some(mut node) = html.tree.get_mut(node_id) {
        node.append_id(a_node_id);
    }
}

/// Inserts self-links for all `<dt>` elements in the given HTML that do not already
/// contain a direct child anchor (`<a>`) element. This function selects all `<dt>`
/// elements that lack an anchor tag and wraps their children with a link pointing
/// to the element’s own `id` attribute.
///
/// # Arguments
///
/// * `html` - A mutable reference to the `Html` structure, representing the HTML
///   document to be processed.
///
/// # Returns
///
/// * `Result<(), DocError>` - Returns `Ok(())` if all operations succeed, otherwise
///   returns a `DocError` if an error is encountered.
pub fn insert_self_links_for_dts(html: &mut Html) -> Result<(), DocError> {
    let selector = Selector::parse("dt:not(:has(a)").unwrap();
    let subs = html.select(&selector).map(|el| el.id()).collect::<Vec<_>>();
    for el_id in subs {
        wrap_children_with_link_to_id(html, el_id);
    }
    Ok(())
}

/// Removes all empty `<p>` elements from the given HTML document. This function
/// selects all `<p>` elements that have no children or content and removes them
/// from the HTML tree structure to clean up any unnecessary empty elements.
///
/// # Arguments
///
/// * `html` - A mutable reference to the `Html` structure, representing the HTML
///   document to be modified.
///
/// # Returns
///
/// * `Result<(), DocError>` - Returns `Ok(())` if all empty `<p>` elements are
///   successfully removed, otherwise returns a `DocError` if an error occurs.
pub fn remove_empty_p(html: &mut Html) -> Result<(), DocError> {
    let selector = Selector::parse("p:empty").unwrap();
    let dels = html.select(&selector).map(|el| el.id()).collect::<Vec<_>>();

    for id in dels {
        html.tree.get_mut(id).unwrap().detach();
    }

    Ok(())
}

/// Adds unique `id` attributes to HTML elements that are missing them.
///
/// This function scans through an HTML document, identifying elements that either:
/// 1. Already contain an `id` attribute, or
/// 2. Lack an `id` attribute but have `data-update-id` attributes or are headers (`<h2>`, `<h3>`) or `<dt>` elements.
///
/// For elements missing `id` attributes, it generates a unique `id` based on the element’s text content,
/// ensuring that the `id` does not conflict with any existing `id`s in the document. If an ID conflict
/// arises, a numeric suffix (e.g., `_2`, `_3`) is appended to the generated `id` until uniqueness is ensured.
///
/// # Arguments
///
/// * `html` - A mutable reference to an HTML document represented by the `Html` type.
///
/// # Returns
///
/// This function returns `Ok(())` on success or a `DocError` if an error occurs.
///
/// # Errors
///
/// If a `DocError` occurs during processing, such as a failure to parse selectors or update attributes,
/// the error is returned.
///
/// # Example
///
/// ```rust
/// # use scraper::Html;
/// # use rari_doc::html::modifier::add_missing_ids;
///
/// let mut html = Html::parse_document("<h2>Some Heading</h2>");
/// add_missing_ids(&mut html);
/// ```
///
/// After calling this function, the HTML will have generated unique `id` attributes for
/// elements without `id`s, based on the element’s content text.
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
                let id = uniquify_id(&mut ids, anchorize(&text));
                (el.id(), id.to_string())
            })
            .collect::<Vec<_>>();
    for (el_id, id) in subs {
        insert_attribute(html, el_id, "id", &id);
        remove_attribute(html, el_id, "data-update-id");
    }
    Ok(())
}
