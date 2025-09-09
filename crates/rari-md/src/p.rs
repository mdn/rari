use comrak::nodes::{AstNode, NodeValue};

use crate::ext::{DELIM_END, DELIM_END_LEN, DELIM_START, DELIM_START_LEN};

fn only_escaped_templ(b: &[u8]) -> bool {
    let b = b.trim_ascii_end().trim_ascii_start();
    if b.starts_with(DELIM_START.as_bytes()) {
        let start = DELIM_START_LEN;
        if let Some(end) = b[start..]
            .windows(DELIM_END_LEN)
            .position(|window| window == DELIM_END.as_bytes())
        {
            if start + end + DELIM_END_LEN == b.len() {
                return true;
            } else {
                return only_escaped_templ(&b[start + end + DELIM_END_LEN..]);
            }
        }
    }
    false
}

pub(crate) fn is_escaped_templ_p<'a>(p: &'a AstNode<'a>) -> bool {
    p.children().all(|child| match &child.data.borrow().value {
        NodeValue::Text(t) => only_escaped_templ(t.as_bytes()),
        NodeValue::SoftBreak => true,
        _ => false,
    })
}

pub(crate) fn is_empty_p<'a>(p: &'a AstNode<'a>) -> bool {
    p.first_child().is_none()
}

pub(crate) fn fix_p<'a>(p: &'a AstNode<'a>) {
    for child in p.reverse_children() {
        p.insert_before(child)
    }
    p.detach();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_only_escaped_templ() {
        let b = "⟬0⟭".as_bytes();
        assert!(only_escaped_templ(b));
        let b = "⟬0⟭⟬1⟭".as_bytes();
        assert!(only_escaped_templ(b));
        let b = "⟬0⟭\n⟬1⟭".as_bytes();
        assert!(only_escaped_templ(b));
        let b = "⟬0⟭ ⟬1⟭".as_bytes();
        assert!(only_escaped_templ(b));
        let b = "⟬0⟭,⟬1⟭".as_bytes();
        assert!(!only_escaped_templ(b));
    }
}
