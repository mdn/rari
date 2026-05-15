use lol_html::html_content::Element;

use crate::pages::page::PageLike;

pub(crate) struct Sourcepos {
    pub line: i64,
    pub col: i64,
    pub end_line: i64,
    pub end_col: i64,
}

impl Sourcepos {
    /// Parse a `"line:col-line:col"` sourcepos string into body-relative
    /// coordinates. Returns `None` if the start position is missing or
    /// malformed; a malformed end position falls back to `(-1, -1)`.
    pub(crate) fn parse(s: &str) -> Option<Self> {
        let (start, end) = s.split_once('-')?;
        let parse_pair = |s: &str| -> Option<(i64, i64)> {
            let (line, col) = s.split_once(':')?;
            let line = line.parse::<i64>().ok().unwrap_or(-1);
            let col = col.parse::<i64>().ok().unwrap_or(0);
            Some((line, col))
        };
        let (line, col) = parse_pair(start)?;
        let (end_line, end_col) = parse_pair(end).unwrap_or((-1, -1));
        Some(Self {
            line,
            col,
            end_line,
            end_col,
        })
    }

    /// Shift `line` and `end_line` by `offset`. `-1` sentinel lines are
    /// left untouched.
    pub(crate) fn shift_lines(mut self, offset: i64) -> Self {
        if self.line != -1 {
            self.line += offset;
        }
        if self.end_line != -1 {
            self.end_line += offset;
        }
        self
    }
}

pub(crate) fn parse_sourcepos(el: &Element, page: &impl PageLike) -> Option<Sourcepos> {
    let raw = el.get_attribute("data-sourcepos")?;
    let offset = i64::try_from(page.fm_offset()).unwrap_or(0);
    Some(Sourcepos::parse(&raw)?.shift_lines(offset))
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use lol_html::{RewriteStrSettings, element, rewrite_str};

    use super::*;
    use crate::test_utils::TestPage;

    // ── Sourcepos::parse — pure string parsing ────────────────────────────

    #[test]
    fn parse_missing_dash_returns_none() {
        assert!(Sourcepos::parse("1:1").is_none());
    }

    #[test]
    fn parse_malformed_start_returns_none() {
        assert!(Sourcepos::parse("1-2:3").is_none());
    }

    #[test]
    fn parse_happy_path() {
        let sp = Sourcepos::parse("3:5-7:9").unwrap();
        assert_eq!((sp.line, sp.col, sp.end_line, sp.end_col), (3, 5, 7, 9));
    }

    #[test]
    fn parse_unparseable_start_line_yields_sentinel() {
        let sp = Sourcepos::parse("abc:5-7:9").unwrap();
        assert_eq!((sp.line, sp.col), (-1, 5));
    }

    #[test]
    fn parse_unparseable_start_col_yields_zero() {
        let sp = Sourcepos::parse("3:abc-7:9").unwrap();
        assert_eq!((sp.line, sp.col), (3, 0));
    }

    #[test]
    fn parse_malformed_end_falls_back_to_sentinels() {
        let sp = Sourcepos::parse("3:5-bogus").unwrap();
        assert_eq!((sp.line, sp.col), (3, 5));
        assert_eq!((sp.end_line, sp.end_col), (-1, -1));
    }

    #[test]
    fn parse_unparseable_end_line_yields_sentinel() {
        let sp = Sourcepos::parse("3:5-xyz:9").unwrap();
        assert_eq!((sp.end_line, sp.end_col), (-1, 9));
    }

    #[test]
    fn parse_unparseable_end_col_yields_zero() {
        let sp = Sourcepos::parse("3:5-7:xyz").unwrap();
        assert_eq!((sp.end_line, sp.end_col), (7, 0));
    }

    // ── Sourcepos::shift_lines ────────────────────────────────────────────

    #[test]
    fn shift_lines_adds_offset_to_both_lines() {
        let sp = Sourcepos {
            line: 3,
            col: 5,
            end_line: 7,
            end_col: 9,
        }
        .shift_lines(10);
        assert_eq!((sp.line, sp.col, sp.end_line, sp.end_col), (13, 5, 17, 9));
    }

    #[test]
    fn shift_lines_leaves_sentinel_lines_untouched() {
        let sp = Sourcepos {
            line: -1,
            col: 0,
            end_line: -1,
            end_col: -1,
        }
        .shift_lines(10);
        assert_eq!((sp.line, sp.end_line), (-1, -1));
    }

    // ── parse_sourcepos wrapper ───────────────────────────────────────────

    fn wrapper(html: &str, fm_offset: usize) -> Option<Sourcepos> {
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
    fn wrapper_missing_attribute_returns_none() {
        assert!(wrapper("<x></x>", 0).is_none());
    }

    #[test]
    fn wrapper_applies_fm_offset() {
        let sp = wrapper(r#"<x data-sourcepos="3:5-7:9"></x>"#, 10).unwrap();
        assert_eq!((sp.line, sp.col, sp.end_line, sp.end_col), (13, 5, 17, 9));
    }
}
