use comrak::nodes::{AstNode, NodeValue};
use itertools::Itertools;

fn only_escaped_templ(b: &[u8], start: usize) -> bool {
    let b = &b[..b
        .iter()
        .rev()
        .find_position(|c| *c != &b'\n')
        .map(|(i, _)| b.len() - i)
        .unwrap_or(0)];
    if b[start..].starts_with("!::::".as_bytes()) {
        if let Some(end) = b[start..]
            .windows(5)
            .position(|window| window == "::::!".as_bytes())
        {
            if start + end + 5 == b.len() {
                return true;
            } else {
                return only_escaped_templ(b, start + end + 5);
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
        let b = b"!::::0::::!";
        assert!(only_escaped_templ(b, 0));
        let b = b"!::::0::::!!::::1::::!";
        assert!(only_escaped_templ(b, 0));
        let b = b"foo !::::0::::!!::::1::::!";
        assert!(!only_escaped_templ(b, 0));
    }
}
