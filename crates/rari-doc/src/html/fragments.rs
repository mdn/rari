use std::collections::HashSet;

use ego_tree::NodeId;
use percent_encoding::percent_decode_str;
use rari_utils::concat_strs;
use scraper::{Html, Selector};

use super::fix_link::parse_sourcepos;
use super::modifier::{insert_attribute, remove_attribute};
use crate::issues::get_issue_counter;
use crate::pages::page::PageLike;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BrokenFragment {
    pub node_id: NodeId,
    pub href: String,
    /// `(line, col, end_line, end_col)` of author-written links; `None` for macro output.
    pub pos: Option<(i64, i64, i64, i64)>,
    pub suggestion: Option<String>,
}

fn is_ignored(href: &str) -> bool {
    let lower = href.to_ascii_lowercase();
    lower == "#"
        || lower == "#top"
        || lower.starts_with("#:~:")
        || lower.starts_with("#livesample_fullscreen=")
}

/// Finds `#fragment` links without a target on the page and the node ids of all fragment links.
pub(crate) fn find_broken_fragments(
    html: &Html,
    fm_offset: usize,
) -> (Vec<BrokenFragment>, Vec<NodeId>) {
    let mut targets: HashSet<&str> = HashSet::new();
    for (selector, attr) in [
        ("*[id]", "id"),
        ("a[name]", "name"),
        (".bc-data[data-query]", "data-query"),
        (".bc-specs[data-bcd-query]", "data-bcd-query"),
    ] {
        let selector = Selector::parse(selector).unwrap();
        targets.extend(
            html.select(&selector)
                .filter_map(|el| el.attr(attr))
                .filter(|v| !v.is_empty()),
        );
    }

    let selector = Selector::parse(r##"a[href^="#"]"##).unwrap();
    let mut broken = vec![];
    let mut all = vec![];
    for el in html.select(&selector) {
        all.push(el.id());
        let Some(href) = el.attr("href") else {
            continue;
        };
        if is_ignored(href) {
            continue;
        }
        let fragment = &href[1..];
        let decoded = percent_decode_str(fragment).decode_utf8_lossy();
        if targets.contains(fragment) || targets.contains(decoded.as_ref()) {
            continue;
        }
        let sourcepos = el.attr("data-sourcepos");
        // Banner l10n strings hardcode this id, which translated pages don't use.
        if sourcepos.is_none() && href == "#browser_compatibility" {
            continue;
        }
        let lower = decoded.to_lowercase();
        let mut candidates = targets.iter().filter(|t| t.to_lowercase() == lower);
        let suggestion = match (candidates.next(), candidates.next()) {
            (Some(id), None) => Some(concat_strs!("#", id)),
            _ => None,
        };
        broken.push(BrokenFragment {
            node_id: el.id(),
            href: href.to_string(),
            pos: sourcepos.and_then(|pos| parse_sourcepos(pos, fm_offset)),
            suggestion,
        });
    }
    (broken, all)
}

/// Reports `#fragment` links whose target doesn't exist on the page, and strips
/// the `data-sourcepos` attribute kept on them by `post_process_html`.
pub fn check_fragment_links(html: &mut Html, page: &impl PageLike, data_issues: bool) {
    let (broken, all) = find_broken_fragments(html, page.fm_offset());
    for broken in broken {
        let ic = get_issue_counter();
        match (broken.pos, broken.suggestion) {
            (Some((line, col, end_line, end_col)), Some(redirect)) => tracing::warn!(
                source = "broken-fragment",
                ic = ic,
                line = line,
                col = col,
                end_line = end_line,
                end_col = end_col,
                url = broken.href,
                redirect = redirect,
            ),
            (Some((line, col, end_line, end_col)), None) => tracing::warn!(
                source = "broken-fragment",
                ic = ic,
                line = line,
                col = col,
                end_line = end_line,
                end_col = end_col,
                url = broken.href,
            ),
            (None, _) => {
                tracing::warn!(source = "templ-broken-fragment", ic = ic, url = broken.href)
            }
        }
        if data_issues {
            insert_attribute(html, broken.node_id, "data-flaw", &ic.to_string());
        }
    }
    for node_id in all {
        remove_attribute(html, node_id, "data-sourcepos");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        name: &'static str,
        html: &'static str,
        /// `(href, has position, suggestion)`
        expected: Vec<(&'static str, bool, Option<&'static str>)>,
    }

    #[test]
    fn finds_broken_fragments() {
        let cases = vec![
            Case {
                name: "valid id",
                html: r##"<h2 id="syntax">S</h2><a href="#syntax" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "missing id",
                html: r##"<a href="#nope" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![("#nope", true, None)],
            },
            Case {
                name: "ill-cased",
                html: r##"<h2 id="syntax">S</h2><a href="#Syntax" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![("#Syntax", true, Some("#syntax"))],
            },
            Case {
                name: "ambiguous case-insensitive match",
                html: r##"<p id="a-b"></p><p id="A-B"></p><a href="#a-B" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![("#a-B", true, None)],
            },
            Case {
                name: "percent-encoded",
                html: r##"<h2 id="caf&#xe9;">S</h2><a href="#caf%C3%A9" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "empty fragment",
                html: r##"<a href="#" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "top",
                html: r##"<a href="#top">x</a><a href="#TOP">y</a>"##,
                expected: vec![],
            },
            Case {
                name: "text directive",
                html: r##"<a href="#:~:text=foo">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "live sample fullscreen",
                html: r##"<a href="#livesample_fullscreen=x">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "a[name] target",
                html: r##"<a name="old"></a><a href="#old" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "multi compat id",
                html: r##"<div class="bc-data" data-query="webassembly.multiMemory" data-multiple="true"></div><a href="#webassembly.multiMemory">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "spec placeholder id",
                html: r##"<div class="bc-specs" data-bcd-query="api.Foo"></div><a href="#api.Foo">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "macro-produced link without sourcepos",
                html: r##"<a href="#nope">x</a>"##,
                expected: vec![("#nope", false, None)],
            },
            Case {
                name: "macro-produced browser_compatibility link",
                html: r##"<a href="#browser_compatibility">x</a>"##,
                expected: vec![],
            },
            Case {
                name: "author-written browser_compatibility link",
                html: r##"<a href="#browser_compatibility" data-sourcepos="2:3-2:20">x</a>"##,
                expected: vec![("#browser_compatibility", true, None)],
            },
            Case {
                name: "non-fragment links ignored",
                html: r##"<a href="/en-US/docs/Foo#bar">x</a>"##,
                expected: vec![],
            },
        ];
        for case in cases {
            let html = Html::parse_fragment(case.html);
            let (broken, _) = find_broken_fragments(&html, 1);
            let actual: Vec<_> = broken
                .iter()
                .map(|b| (b.href.as_str(), b.pos.is_some(), b.suggestion.as_deref()))
                .collect();
            assert_eq!(actual, case.expected, "{}", case.name);
        }
    }

    #[test]
    fn positions_include_frontmatter_offset() {
        let html = Html::parse_fragment(r##"<a href="#nope" data-sourcepos="2:3-2:20">x</a>"##);
        let (broken, _) = find_broken_fragments(&html, 4);
        assert_eq!(broken[0].pos, Some((6, 3, 6, 20)));
    }

    #[test]
    fn strips_sourcepos_from_fragment_links() {
        let mut html = Html::parse_fragment(
            r##"<h2 id="a">A</h2><a href="#a" data-sourcepos="1:1-1:5">x</a><a href="#b" data-sourcepos="1:1-1:5">y</a>"##,
        );
        let (_, all) = find_broken_fragments(&html, 0);
        assert_eq!(all.len(), 2);
        for id in all {
            remove_attribute(&mut html, id, "data-sourcepos");
        }
        assert!(!html.html().contains("data-sourcepos"));
    }
}
