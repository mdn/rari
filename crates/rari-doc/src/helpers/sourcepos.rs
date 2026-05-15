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
