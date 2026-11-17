//! Post-processed sidebars that are reused across pages.
//!
//! Post-processing a sidebar is expensive and, apart from marking the current page, yields the
//! same HTML for every page of a locale. A [`SidebarTemplate`] holds that shared HTML together
//! with the insertions that mark a given page as current.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, OnceLock};

use dashmap::DashMap;
use ego_tree::NodeId;
use indexmap::IndexSet;
use rari_types::locale::Locale;
use regex::Regex;
use scraper::{Html, Node, Selector};
use sha2::{Digest, Sha256};

use super::modifier::insert_attribute;
use super::rewriter::post_process_html_internal;
use crate::error::DocError;
use crate::pages::page::PageLike;

/// Attribute marking an element whose content gets wrapped in `<em>` for the current page.
pub(crate) const EM_MARKER_ATTR: &str = "data-sb-em";
/// Enclosing the content of an `EM_MARKER_ATTR` element, as `<!--sb-em:N-->` and `<!--/sb-em:N-->`.
pub(crate) const EM_MARKER_COMMENT: &str = "sb-em:";
/// Attribute marking a `<details>` that gets opened for the current page.
const OPEN_MARKER_ATTR: &str = "data-sb-open";
/// Attribute marking where `aria-current` goes if the page's URL equals the recorded one.
pub(crate) const CURRENT_MARKER_ATTR: &str = "data-sb-current";

const MARKERS: [&str; 4] = [
    EM_MARKER_ATTR,
    EM_MARKER_COMMENT,
    OPEN_MARKER_ATTR,
    CURRENT_MARKER_ATTR,
];

static LINK_SELECTOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a[href]").unwrap());
static MARKER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(constcat::concat!(
        "<!--(?<em_end>/?)",
        EM_MARKER_COMMENT,
        r"(?<em>\d+)-->",
        "| ",
        OPEN_MARKER_ATTR,
        r#"="(?<open>\d+)""#,
        "| ",
        CURRENT_MARKER_ATTR,
        r#"="(?<current>\d+)""#,
    ))
    .unwrap()
});

type TemplateKey = (Locale, [u8; 32]);
type TemplateCache = DashMap<TemplateKey, Arc<OnceLock<Option<Arc<SidebarTemplate>>>>>;

static TEMPLATES: LazyLock<TemplateCache> = LazyLock::new(DashMap::new);

/// Records the page-specific parts of post-processing, so the output can be reused across pages.
#[derive(Default)]
pub(crate) struct PageRecorder {
    /// Resolved URLs of the internal links, indexed by their `CURRENT_MARKER_ATTR` value.
    pub(crate) link_urls: RefCell<Vec<String>>,
    /// Whether the output depends on the page in a way that is not recorded.
    pub(crate) page_specific: Cell<bool>,
}

pub(crate) struct SidebarTemplate {
    html: String,
    /// The insertions into `html` marking a page as current, by page URL and sorted by offset.
    insertions: HashMap<String, Vec<(usize, &'static str)>>,
}

/// What marks the first link to a URL as current: its parent and the `<details>` around it.
struct LinkMark {
    href: String,
    em: Option<usize>,
    open: Vec<usize>,
}

/// The marks added by `mark_links`, with the number of em and open markers.
struct Marks {
    links: Vec<LinkMark>,
    em_count: usize,
    open_count: usize,
}

impl SidebarTemplate {
    /// Returns the template for a rendered sidebar, or `None` if its post-processed HTML
    /// depends on the page in more than the current page marks.
    pub(crate) fn get_or_build<T: PageLike>(
        rendered_sidebar: &str,
        page: &T,
    ) -> Result<Option<Arc<Self>>, DocError> {
        let key = (page.locale(), Sha256::digest(rendered_sidebar).into());
        // Avoid the write lock of `entry` on hits, as many pages share a sidebar.
        let cell = TEMPLATES
            .get(&key)
            .map(|cell| cell.clone())
            .unwrap_or_else(|| TEMPLATES.entry(key).or_default().clone());
        if let Some(template) = cell.get() {
            return Ok(template.clone());
        }
        let template = Self::build(rendered_sidebar, page)?.map(Arc::new);
        // Racing builds are identical, so it does not matter which one wins.
        Ok(cell.get_or_init(|| template).clone())
    }

    fn build<T: PageLike>(rendered_sidebar: &str, page: &T) -> Result<Option<Self>, DocError> {
        if MARKERS
            .iter()
            .any(|marker| rendered_sidebar.contains(marker))
        {
            return Ok(None);
        }
        let mut fragment = Html::parse_fragment(rendered_sidebar);
        let Some(Marks {
            links,
            em_count,
            open_count,
        }) = mark_links(&mut fragment)
        else {
            return Ok(None);
        };

        let recorder = PageRecorder::default();
        let marked_html =
            post_process_html_internal(&fragment.html(), page, true, Some(&recorder))?;
        if recorder.page_specific.get() {
            return Ok(None);
        }
        let link_urls = recorder.link_urls.into_inner();

        // Strip the markers, keeping their offsets in the stripped HTML.
        let mut html = String::with_capacity(marked_html.len());
        let mut em_starts = HashMap::new();
        let mut em_ends = HashMap::new();
        let mut opens = HashMap::new();
        let mut currents = HashMap::new();
        let mut last = 0;
        for captures in MARKER.captures_iter(&marked_html) {
            let marker = captures.get(0).unwrap();
            html.push_str(&marked_html[last..marker.start()]);
            last = marker.end();
            let (offsets, index) = if let Some(index) = captures.name("em") {
                if captures["em_end"].is_empty() {
                    (&mut em_starts, index)
                } else {
                    (&mut em_ends, index)
                }
            } else if let Some(index) = captures.name("open") {
                (&mut opens, index)
            } else {
                (&mut currents, captures.name("current").unwrap())
            };
            let Ok(index) = index.as_str().parse::<usize>() else {
                return Ok(None);
            };
            if offsets.insert(index, html.len()).is_some() {
                return Ok(None);
            }
        }
        html.push_str(&marked_html[last..]);
        // Lookalikes, e.g. from entity-encoded text, would otherwise be stripped as markers.
        if em_starts.len() != em_count
            || em_ends.len() != em_count
            || opens.len() != open_count
            || currents.len() != link_urls.len()
        {
            return Ok(None);
        }

        let mut insertions: HashMap<String, Vec<(usize, &'static str)>> = HashMap::new();
        for LinkMark { href, em, open } in links {
            let insertions = insertions.entry(href).or_default();
            if let Some(em) = em {
                let (Some(start), Some(end)) = (em_starts.get(&em), em_ends.get(&em)) else {
                    return Ok(None);
                };
                insertions.push((*start, "<em>"));
                insertions.push((*end, "</em>"));
            }
            for open in open {
                let Some(offset) = opens.get(&open) else {
                    return Ok(None);
                };
                insertions.push((*offset, " open=\"\""));
            }
        }
        for (index, url) in link_urls.into_iter().enumerate() {
            let Some(offset) = currents.get(&index) else {
                return Ok(None);
            };
            insertions
                .entry(url)
                .or_default()
                .push((*offset, " aria-current=\"page\""));
        }
        for insertions in insertions.values_mut() {
            insertions.sort_by_key(|(offset, _)| *offset);
        }

        Ok(Some(Self { html, insertions }))
    }

    /// Returns the sidebar's HTML with the page at `url` marked as current.
    pub(crate) fn render(&self, url: &str) -> String {
        let Some(insertions) = self.insertions.get(url) else {
            return self.html.clone();
        };
        let extra: usize = insertions.iter().map(|(_, text)| text.len()).sum();
        let mut out = String::with_capacity(self.html.len() + extra);
        let mut last = 0;
        for (offset, text) in insertions {
            out.push_str(&self.html[last..*offset]);
            out.push_str(text);
            last = *offset;
        }
        out.push_str(&self.html[last..]);
        out
    }
}

/// Adds marker attributes for everything `expand_details_and_mark_current` could touch, for any
/// page URL. Returns `None` if the marks cannot represent the sidebar.
fn mark_links(fragment: &mut Html) -> Option<Marks> {
    let mut em_ids: IndexSet<NodeId> = IndexSet::new();
    let mut open_ids: IndexSet<NodeId> = IndexSet::new();
    let mut link_marks: Vec<LinkMark> = vec![];
    let mut seen = HashSet::new();
    for link in fragment.select(&LINK_SELECTOR) {
        let href = link.value().attr("href").unwrap_or_default();
        // Only the first link to a URL is marked.
        if !seen.insert(href) {
            continue;
        }
        let em = link
            .parent()
            .filter(|parent| parent.value().is_element())
            .map(|parent| em_ids.insert_full(parent.id()).0);
        let mut open = vec![];
        for ancestor in link.ancestors() {
            if let Node::Element(el) = ancestor.value()
                && el.name() == "details"
            {
                match el.attr("open") {
                    None => open.push(open_ids.insert_full(ancestor.id()).0),
                    Some("") => {}
                    Some(_) => return None,
                }
            }
        }
        link_marks.push(LinkMark {
            href: href.to_string(),
            em,
            open,
        });
    }
    let (em_count, open_count) = (em_ids.len(), open_ids.len());
    for (index, id) in em_ids.into_iter().enumerate() {
        insert_attribute(fragment, id, EM_MARKER_ATTR, &index.to_string());
    }
    for (index, id) in open_ids.into_iter().enumerate() {
        insert_attribute(fragment, id, OPEN_MARKER_ATTR, &index.to_string());
    }
    Some(Marks {
        links: link_marks,
        em_count,
        open_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::sidebar::postprocess_sidebar_uncached;
    use crate::test_utils::TestPage;

    const SIDEBAR: &str = concat!(
        "<ol>",
        "<li class=\"section\"><a data-templ-link href=\"/en-US/play/A\">A</a></li>",
        "<li><details><summary><a data-templ-link href=\"/en-US/play/B\"><code>B.b</code></a></summary><ol>",
        "<li><a data-templ-link href=\"/en-US/play/B/C\">C</a></li>",
        "<li><details open><summary>D</summary><ol>",
        "<li><a data-templ-link href=\"/en-US/play/B/D\">D</a></li>",
        "<li><a data-templ-link href=\"/en-US/play/A\">A again</a></li>",
        "</ol></details></li>",
        "</ol></details></li>",
        "<li><a data-templ-link href=\"/en-US/play/E\">E</a> <a data-templ-link href=\"/en-US/play/F\">F</a></li>",
        "<li><details><a data-templ-link href=\"/en-US/play/G\">G</a></details></li>",
        "<li><a data-templ-link href=\"https://example.com/\">External</a></li>",
        "</ol>",
    );

    #[test]
    fn template_matches_uncached_post_processing() {
        struct Case {
            name: &'static str,
            url: &'static str,
        }
        let cases = vec![
            Case {
                name: "page not in sidebar",
                url: "/en-US/play/Other",
            },
            Case {
                name: "top-level link, linked twice",
                url: "/en-US/play/A",
            },
            Case {
                name: "link in summary",
                url: "/en-US/play/B",
            },
            Case {
                name: "link in closed details",
                url: "/en-US/play/B/C",
            },
            Case {
                name: "link in nested open details",
                url: "/en-US/play/B/D",
            },
            Case {
                name: "first link of shared parent",
                url: "/en-US/play/E",
            },
            Case {
                name: "second link of shared parent",
                url: "/en-US/play/F",
            },
            Case {
                name: "link directly in details",
                url: "/en-US/play/G",
            },
        ];
        // The links stay in en-US, as broken links in other locales need l10n data.
        for locale in [Locale::EnUs, Locale::De] {
            let template = SidebarTemplate::build(
                SIDEBAR,
                &TestPage {
                    locale,
                    ..Default::default()
                },
            )
            .unwrap()
            .expect("sidebar is page-independent");

            for case in &cases {
                let page = TestPage {
                    url: case.url.to_string(),
                    locale,
                    ..Default::default()
                };
                assert_eq!(
                    template.render(case.url),
                    postprocess_sidebar_uncached(SIDEBAR, &page).unwrap(),
                    "{} ({locale})",
                    case.name
                );
            }

            let current = template.render("/en-US/play/B/C");
            for mark in ["<em>", "<details open=\"\">", "aria-current=\"page\""] {
                assert!(current.contains(mark), "{mark} is missing in {current}");
            }
        }
    }

    #[test]
    fn unsupported_sidebars_have_no_template() {
        struct Case {
            name: &'static str,
            sidebar: &'static str,
        }
        let cases = vec![
            Case {
                name: "unchecked internal link",
                sidebar: "<ol><li><a href=\"/en-US/play/A\">A</a></li></ol>",
            },
            Case {
                name: "image",
                sidebar: "<ol><li><img src=\"a.png\"></li></ol>",
            },
            Case {
                name: "details with non-empty open",
                sidebar: "<ol><li><details open=\"open\"><a data-templ-link href=\"/en-US/play/A\">A</a></details></li></ol>",
            },
            Case {
                name: "marker lookalike",
                sidebar: "<ol><li data-sb-em=\"0\">A</li></ol>",
            },
            Case {
                name: "entity-encoded marker lookalike",
                sidebar: "<ol><li>A data-sb&#45;open=&quot;9&quot;</li></ol>",
            },
        ];
        for case in cases {
            assert!(
                SidebarTemplate::build(case.sidebar, &TestPage::default())
                    .unwrap()
                    .is_none(),
                "{}",
                case.name
            );
        }
    }
}
