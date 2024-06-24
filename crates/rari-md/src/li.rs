use comrak::nodes::{AstNode, NodeValue};

pub(crate) fn remove_p<'a>(list: &'a AstNode<'a>) {
    for child in list.children() {
        if let Some(i) = child.first_child() {
            if !matches!(i.data.borrow().value, NodeValue::Paragraph) {
                continue;
            }
            i.detach();
            if let Some(new_first) = child.first_child() {
                for sub in i.children() {
                    new_first.insert_before(sub);
                }
            } else {
                for sub in i.children() {
                    child.append(sub);
                }
            }
        }
    }
}
