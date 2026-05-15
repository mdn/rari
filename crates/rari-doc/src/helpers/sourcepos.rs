use lol_html::html_content::Element;

use crate::pages::page::PageLike;

pub(crate) struct Sourcepos {
    pub line: i64,
    pub col: i64,
    pub end_line: i64,
    pub end_col: i64,
}

pub(crate) fn parse_sourcepos(el: &Element, page: &impl PageLike) -> Option<Sourcepos> {
    let pos = el.get_attribute("data-sourcepos")?;
    let (start, end) = pos.split_once('-')?;
    let fm_offset = page.fm_offset();
    let parse_pair = |s: &str| -> Option<(i64, i64)> {
        let (line, col) = s.split_once(':')?;
        let line = line
            .parse::<i64>()
            .map(|l| l + i64::try_from(fm_offset).unwrap_or(0))
            .ok()
            .unwrap_or(-1);
        let col = col.parse::<i64>().ok().unwrap_or(0);
        Some((line, col))
    };
    let (line, col) = parse_pair(start)?;
    let (end_line, end_col) = parse_pair(end).unwrap_or((-1, -1));
    Some(Sourcepos {
        line,
        col,
        end_line,
        end_col,
    })
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use lol_html::{RewriteStrSettings, element, rewrite_str};

    use super::*;
    use crate::test_utils::TestPage;

    fn parse(html: &str, fm_offset: usize) -> Option<Sourcepos> {
        let page = TestPage {
            fm_offset,
            ..Default::default()
        };
        let captured: RefCell<Option<Option<Sourcepos>>> = RefCell::new(None);
        rewrite_str(
            html,
            RewriteStrSettings {
                element_content_handlers: vec![element!("x", |el| {
                    *captured.borrow_mut() = Some(parse_sourcepos(el, &page));
                    Ok(())
                })],
                ..Default::default()
            },
        )
        .unwrap();
        captured.into_inner().expect("handler did not run")
    }

    #[test]
    fn missing_attribute_returns_none() {
        assert!(parse("<x></x>", 0).is_none());
    }

    #[test]
    fn missing_dash_separator_returns_none() {
        assert!(parse(r#"<x data-sourcepos="1:1"></x>"#, 0).is_none());
    }

    #[test]
    fn malformed_start_returns_none() {
        // No ':' in the start position.
        assert!(parse(r#"<x data-sourcepos="1-2:3"></x>"#, 0).is_none());
    }

    #[test]
    fn happy_path_no_offset() {
        let sp = parse(r#"<x data-sourcepos="3:5-7:9"></x>"#, 0).unwrap();
        assert_eq!((sp.line, sp.col, sp.end_line, sp.end_col), (3, 5, 7, 9));
    }

    #[test]
    fn fm_offset_applied_to_both_lines() {
        let sp = parse(r#"<x data-sourcepos="3:5-7:9"></x>"#, 10).unwrap();
        assert_eq!((sp.line, sp.col, sp.end_line, sp.end_col), (13, 5, 17, 9));
    }

    #[test]
    fn unparseable_start_line_yields_sentinel() {
        let sp = parse(r#"<x data-sourcepos="abc:5-7:9"></x>"#, 10).unwrap();
        // Line falls back to -1 (no offset applied); col still parses.
        assert_eq!((sp.line, sp.col), (-1, 5));
    }

    #[test]
    fn unparseable_start_col_yields_zero() {
        let sp = parse(r#"<x data-sourcepos="3:abc-7:9"></x>"#, 0).unwrap();
        assert_eq!((sp.line, sp.col), (3, 0));
    }

    #[test]
    fn malformed_end_falls_back_to_sentinels() {
        // End has no ':'.
        let sp = parse(r#"<x data-sourcepos="3:5-bogus"></x>"#, 10).unwrap();
        // Start is parsed normally; end gets (-1, -1).
        assert_eq!((sp.line, sp.col), (13, 5));
        assert_eq!((sp.end_line, sp.end_col), (-1, -1));
    }

    #[test]
    fn unparseable_end_line_yields_sentinel() {
        let sp = parse(r#"<x data-sourcepos="3:5-xyz:9"></x>"#, 10).unwrap();
        assert_eq!((sp.end_line, sp.end_col), (-1, 9));
    }

    #[test]
    fn unparseable_end_col_yields_zero() {
        let sp = parse(r#"<x data-sourcepos="3:5-7:xyz"></x>"#, 10).unwrap();
        assert_eq!((sp.end_line, sp.end_col), (17, 0));
    }
}
