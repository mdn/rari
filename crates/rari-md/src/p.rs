use comrak::nodes::{AstNode, NodeValue};

use crate::ext::{DELIM_END, DELIM_START};

fn only_escaped_templ(b: &[u8], start: usize) -> bool {
    let b = b.trim_ascii_end();
    if b[start..].starts_with(DELIM_START.as_bytes()) {
        let start = start + DELIM_START.len();
        if let Some(end) = b[start..]
            .windows(DELIM_END.len())
            .position(|window| window == DELIM_END.as_bytes())
        {
            if start + end + DELIM_END.len() == b.len() {
                return true;
            } else {
                return only_escaped_templ(b, start + end + DELIM_END.len());
            }
        }
    }
    false
}

pub(crate) fn is_escaped_templ_p<'a>(p: &'a AstNode<'a>) -> bool {
    if p.children().count() == 1 {
        if let Some(k) = p.first_child() {
            if let NodeValue::Text(t) = &k.data.borrow().value {
                return only_escaped_templ(t.as_bytes(), 0);
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_only_escaped_templ() {
        let b = "⟬0⟭".as_bytes();
        assert!(only_escaped_templ(b, 0));
        let b = "⟬0⟭⟬1⟭".as_bytes();
        assert!(only_escaped_templ(b, 0));
        let b = "⟬0⟭,⟬1⟭".as_bytes();
        assert!(!only_escaped_templ(b, 0));
    }
}
