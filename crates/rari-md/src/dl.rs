use comrak::Arena;
use comrak::nodes::{AstNode, NodeDescriptionItem, NodeValue};

pub(crate) fn is_dl<'a>(list: &'a AstNode<'a>) -> bool {
    list.children().all(|child| {
        if child.children().count() < 2 {
            return false;
        }
        let last_child = child.last_child().unwrap();
        if !matches!(last_child.data.borrow().value, NodeValue::List(_)) {
            return false;
        }
        last_child.children().all(|item| {
            if let Some(i) = item.first_child() {
                if !matches!(i.data.borrow().value, NodeValue::Paragraph) {
                    return false;
                }
                if let Some(j) = i.first_child()
                    && let NodeValue::Text(ref t) = j.data.borrow().value
                {
                    return t.starts_with(": ");
                }
            }
            false
        })
    })
}

pub(crate) fn convert_dl<'a>(arena: &'a Arena<'a>, list: &'a AstNode<'a>) {
    list.data.borrow_mut().value = NodeValue::DescriptionList;
    for child in list.children() {
        child.data.borrow_mut().value = NodeValue::DescriptionTerm;
        let last_child = child.last_child().unwrap();
        if !matches!(last_child.data.borrow().value, NodeValue::List(_)) {
            continue;
        }
        last_child.detach();

        let item = arena.alloc(NodeValue::DescriptionItem(NodeDescriptionItem::default()).into());
        child.insert_before(item);
        child.detach();
        item.append(child);

        for details in last_child.children() {
            if let Some(i) = details.first_child() {
                if !matches!(i.data.borrow().value, NodeValue::Paragraph) {
                    break;
                }
                if let Some(j) = i.first_child()
                    && let NodeValue::Text(ref mut t) = j.data.borrow_mut().value
                {
                    let skip = t.len().min(2);
                    if skip > 0 {
                        *t = t[skip..].to_string().into();
                    }
                }
            }
            details.data.borrow_mut().value = NodeValue::DescriptionDetails;
            details.detach();
            item.append(details);
        }
    }
}
