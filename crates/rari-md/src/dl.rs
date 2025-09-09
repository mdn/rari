use comrak::nodes::{AstNode, NodeValue};

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
                if let Some(j) = i.first_child() {
                    if let NodeValue::Text(ref t) = j.data.borrow().value {
                        //println!("{:?}", std::str::from_utf8(t));
                        return t.starts_with(": ");
                    }
                }
            }
            false
        })
    })
}

pub(crate) fn convert_dl<'a>(list: &'a AstNode<'a>) {
    list.data.borrow_mut().value = NodeValue::DescriptionList;
    for child in list.children() {
        child.data.borrow_mut().value = NodeValue::DescriptionTerm;
        let last_child = child.last_child().unwrap();
        if !matches!(last_child.data.borrow().value, NodeValue::List(_)) {
            continue;
        }
        last_child.detach();
        for item in last_child.reverse_children() {
            if let Some(i) = item.first_child() {
                if !matches!(i.data.borrow().value, NodeValue::Paragraph) {
                    break;
                }
                if let Some(j) = i.first_child() {
                    if let NodeValue::Text(ref mut t) = j.data.borrow_mut().value {
                        match t.len() {
                            0 => {}
                            1 => {
                                t.drain(0..1);
                            }
                            _ => {
                                t.drain(0..2);
                            }
                        }
                    }
                }
            }
            item.data.borrow_mut().value = NodeValue::DescriptionDetails;
            item.detach();
            child.insert_after(item);
        }
    }
}
