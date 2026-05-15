use lol_html::html_content::Element;

use crate::pages::page::PageLike;

pub(crate) struct Sourcepos {
    pub line: i64,
    pub col: i64,
    pub end_line: i64,
    pub end_col: i64,
}

pub(crate) fn parse_sourcepos(el: &mut Element, page: &impl PageLike) -> Option<Sourcepos> {
    let pos = el.get_attribute("data-sourcepos")?;
    let (start, end) = pos.split_once('-')?;
    let (line, col) = start.split_once(':')?;
    let line = line
        .parse::<i64>()
        .map(|l| l + i64::try_from(page.fm_offset()).unwrap_or(l - 1))
        .ok()
        .unwrap_or(-1);
    let col = col.parse::<i64>().ok().unwrap_or(0);
    let (end_line, end_col) = end
        .split_once(':')
        .map(|(end_line, end_col)| {
            let end_line = end_line
                .parse::<i64>()
                .map(|l| l + i64::try_from(page.fm_offset()).unwrap_or(l - 1))
                .ok()
                .unwrap_or(-1);
            let end_col = end_col.parse::<i64>().ok().unwrap_or(0);
            (end_line, end_col)
        })
        .unwrap_or((-1, -1));
    Some(Sourcepos {
        line,
        col,
        end_line,
        end_col,
    })
}
