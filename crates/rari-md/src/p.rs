use comrak::nodes::{AstNode, NodeValue};

fn only_ks(b: &[u8], start: usize) -> bool {
    if b[start..].starts_with("{{".as_bytes()) {
        if let Some(end) = b[start..]
            .windows(2)
            .position(|window| window == "}}".as_bytes())
        {
            if start + end + 2 == b.len() {
                return true;
            } else {
                return only_ks(b, start + end + 2);
            }
        }
    }
    false
}

pub(crate) fn is_ksp<'a>(p: &'a AstNode<'a>) -> bool {
    if p.children().count() == 1 {
        if let Some(k) = p.first_child() {
            if let NodeValue::Text(t) = &k.data.borrow().value {
                return only_ks(t.as_bytes(), 0);
            }
        }
    }
    false
}

pub(crate) fn is_empty_p<'a>(p: &'a AstNode<'a>) -> bool {
    p.first_child().is_none()
}

pub(crate) fn fix_p<'a>(p: &'a AstNode<'a>) {
    if let Some(child) = p.first_child() {
        p.insert_before(child)
    }
    p.detach();
}
