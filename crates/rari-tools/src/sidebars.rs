use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use const_format::concatcp;
use pretty_yaml::config::{FormatOptions, LanguageOptions};
use rari_doc::html::sidebar::{
    BasicEntry, CoreEntry, Sidebar, SidebarEntry, SubPageEntry, SubPageGroupedEntry, WebExtApiEntry,
};
use rari_types::globals::content_root;
use rari_types::locale::{default_locale, Locale};
use rari_utils::concat_strs;

use crate::error::ToolError;
use crate::redirects::{read_redirects_raw, redirects_path};

const PREFIX: &str = "# Do not add comments to this file. They will be lost.\n\n";
static EN_US_DOCS_PREFIX: &str = concatcp!("/", default_locale().as_url_str(), "/docs");

type Pair<'a> = (Cow<'a, str>, Option<Cow<'a, str>>);
type Pairs<'a> = &'a [Pair<'a>];

pub fn sync_sidebars() -> Result<(), ToolError> {
    let mut redirects = HashMap::new();
    let path = redirects_path(Locale::default())?;
    redirects.extend(read_redirects_raw(&path)?);
    let pairs = redirects
        .iter()
        .map(|(from, to)| {
            (
                from.strip_prefix(EN_US_DOCS_PREFIX)
                    .map(Cow::Borrowed)
                    .unwrap_or(Cow::Borrowed(from)),
                Some(
                    to.strip_prefix(EN_US_DOCS_PREFIX)
                        .map(Cow::Borrowed)
                        .unwrap_or(Cow::Borrowed(to)),
                ),
            )
        })
        .collect::<Vec<_>>();
    update_sidebars(&pairs)?;
    Ok(())
}

fn traverse_and_extract_l10nable<'a, 'b: 'a>(entry: &'a SidebarEntry) -> HashSet<&'a str> {
    let (title, hash, children) = match entry {
        SidebarEntry::Section(basic_entry) => (
            basic_entry.core.title.as_deref(),
            basic_entry.core.hash.as_deref(),
            Some(&basic_entry.children),
        ),
        SidebarEntry::ListSubPages(sub_page_entry) => (
            sub_page_entry.core.title.as_deref(),
            sub_page_entry.core.hash.as_deref(),
            None,
        ),
        SidebarEntry::ListSubPagesGrouped(sub_page_entry) => (
            sub_page_entry.core.title.as_deref(),
            sub_page_entry.core.hash.as_deref(),
            None,
        ),
        SidebarEntry::WebExtApi(web_ext_api_entry) => {
            (Some(web_ext_api_entry.title.as_str()), None, None)
        }
        SidebarEntry::Default(basic_entry) => (
            basic_entry.core.title.as_deref(),
            basic_entry.core.hash.as_deref(),
            Some(&basic_entry.children),
        ),
        _ => (None, None, None),
    };
    let mut set: HashSet<_> = if let Some(children) = children {
        children
            .iter()
            .flat_map(traverse_and_extract_l10nable)
            .collect()
    } else {
        Default::default()
    };
    if let Some(hash) = hash {
        set.insert(hash);
    }
    if let Some(title) = title {
        set.insert(title);
    }
    set
}

fn consolidate_l10n(sidebar: &mut Sidebar) {
    let titles = sidebar
        .sidebar
        .iter()
        .flat_map(traverse_and_extract_l10nable)
        .collect::<HashSet<_>>();
    for (_, map) in sidebar.l10n.l10n.iter_mut() {
        map.retain(|k, _| titles.contains(k.as_str()));
    }
}

pub fn fmt_sidebars() -> Result<(), ToolError> {
    for (path, mut sidebar) in read_sidebars()? {
        consolidate_l10n(&mut sidebar);
        write_sidebar(&sidebar, &path)?;
    }
    Ok(())
}

pub(crate) fn update_sidebars(pairs: Pairs<'_>) -> Result<(), ToolError> {
    let sidebars = read_sidebars()?;

    // add leading slash to pairs, because that is what the sidebars use
    let pairs = &pairs
        .iter()
        .map(|(from, to)| {
            let from = if from.starts_with('/') {
                Cow::Borrowed(from.as_ref())
            } else {
                Cow::Owned(concat_strs!("/", from))
            };
            let to = to.as_ref().map(|to| {
                if to.starts_with('/') {
                    Cow::Borrowed(to.as_ref())
                } else {
                    Cow::Owned(concat_strs!("/", to))
                }
            });
            (from, to)
        })
        .collect::<Vec<Pair<'_>>>();

    // Walk the sidebars and potentially replace the links.
    // `process_entry`` is called recursively to process all children
    for (path, mut parsed_sidebar) in sidebars {
        // Store a clone to detect changes later
        let original = parsed_sidebar.clone();
        let entries = parsed_sidebar
            .sidebar
            .into_iter()
            .map(|entry| process_entry(entry, pairs))
            .collect();

        // If the sidebar contents have changed, write it back to the file.
        if original.sidebar != entries {
            parsed_sidebar.sidebar = entries;
            write_sidebar(&parsed_sidebar, &path)?;
        }
    }

    Ok(())
}

fn write_sidebar(sidebar: &Sidebar, path: &Path) -> Result<(), ToolError> {
    let y = serde_yaml_ng::to_string(sidebar)?;
    // Format yaml a bit prettier than serde does
    let y = pretty_yaml::format_text(
        &y,
        &FormatOptions {
            language: LanguageOptions {
                quotes: pretty_yaml::config::Quotes::ForceDouble,
                indent_block_sequence_in_map: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )?;
    let yaml = concat_strs!(PREFIX, &y);
    fs::write(path, &yaml)?;
    Ok(())
}

fn read_sidebars() -> Result<Vec<(std::path::PathBuf, Sidebar)>, ToolError> {
    // read all sidebars
    let mut path = content_root().to_path_buf();
    path.push("sidebars");
    let entries = fs::read_dir(&path)?;

    // map and parse sidebars into a vector of (path, Sidebar)
    entries
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                let path = entry.path();
                if path.is_file()
                    && path
                        .extension()
                        .map(|ex| ex.to_string_lossy() == "yaml")
                        .unwrap_or_default()
                {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .map(|path| {
            let content = fs::read_to_string(&path)?;
            let sidebar: Sidebar = serde_yaml_ng::from_str(&content)?;
            Ok((path, sidebar))
        })
        .collect()
}

fn replace_pairs(link: Option<String>, pairs: Pairs<'_>) -> Option<String> {
    match link {
        Some(link) => {
            let mut has_prefix = false;
            let link = if let Some(l) = link.strip_prefix(EN_US_DOCS_PREFIX) {
                has_prefix = true;
                l.to_string()
            } else {
                link
            };
            for (from, to) in pairs {
                if link == *from {
                    if let Some(to) = to {
                        if has_prefix {
                            return Some(concat_strs!(EN_US_DOCS_PREFIX, to));
                        } else {
                            return Some(to.to_string());
                        }
                    } else {
                        return None;
                    }
                }
            }
            Some(link)
        }
        None => None,
    }
}

fn process_basic_entry(
    BasicEntry {
        core:
            CoreEntry {
                link,
                hash,
                title,
                details,
                code,
            },
        children,
    }: BasicEntry,
    pairs: Pairs<'_>,
) -> Option<BasicEntry> {
    let new_link: Option<String> = replace_pairs(link.clone(), pairs);
    if link.is_some() && new_link.is_none() {
        return None;
    }
    Some(BasicEntry {
        core: CoreEntry {
            link: new_link,
            hash,
            title,
            details,
            code,
        },
        children: children
            .into_iter()
            .map(|c| process_entry(c, pairs))
            .collect(),
    })
}

fn process_sub_page_grouped_entry(
    SubPageGroupedEntry {
        core:
            CoreEntry {
                link,
                hash,
                title,
                details,
                code,
            },
        tags,
        path,
        include_parent,
        depth,
    }: SubPageGroupedEntry,
    pairs: Pairs<'_>,
) -> Option<SubPageGroupedEntry> {
    let new_path: String = replace_pairs(Some(path), pairs)?;
    Some(SubPageGroupedEntry {
        core: CoreEntry {
            link: replace_pairs(link.clone(), pairs),
            hash,
            title,
            details,
            code,
        },
        tags,
        path: new_path,
        include_parent,
        depth,
    })
}
fn process_sub_page_entry(
    SubPageEntry {
        core:
            CoreEntry {
                link,
                hash,
                title,
                details,
                code,
            },
        tags,
        path,
        include_parent,
        depth,
        nested,
    }: SubPageEntry,
    pairs: Pairs<'_>,
) -> Option<SubPageEntry> {
    let new_path: String = replace_pairs(Some(path), pairs)?;
    Some(SubPageEntry {
        core: CoreEntry {
            link: replace_pairs(link.clone(), pairs),
            hash,
            title,
            details,
            code,
        },
        tags,
        path: new_path,
        include_parent,
        depth,
        nested,
    })
}

fn process_entry(entry: SidebarEntry, pairs: Pairs<'_>) -> SidebarEntry {
    match entry {
        SidebarEntry::Section(basic_entry) => {
            if let Some(entry) = process_basic_entry(basic_entry, pairs) {
                SidebarEntry::Section(entry)
            } else {
                SidebarEntry::None
            }
        }
        SidebarEntry::Default(basic_entry) => {
            if let Some(entry) = process_basic_entry(basic_entry, pairs) {
                SidebarEntry::Default(entry)
            } else {
                SidebarEntry::None
            }
        }
        SidebarEntry::ListSubPages(sub_page_entry) => {
            if let Some(entry) = process_sub_page_entry(sub_page_entry, pairs) {
                SidebarEntry::ListSubPages(entry)
            } else {
                SidebarEntry::None
            }
        }
        SidebarEntry::ListSubPagesGrouped(sub_page_entry) => {
            if let Some(entry) = process_sub_page_grouped_entry(sub_page_entry, pairs) {
                SidebarEntry::ListSubPagesGrouped(entry)
            } else {
                SidebarEntry::None
            }
        }

        SidebarEntry::Link(link) => {
            let new_link: Option<String> = replace_pairs(Some(link), pairs);
            if new_link.is_none() {
                return SidebarEntry::None;
            }
            SidebarEntry::Link(new_link.unwrap())
        }
        SidebarEntry::WebExtApi(WebExtApiEntry { title }) => {
            SidebarEntry::WebExtApi(WebExtApiEntry { title })
        }
        SidebarEntry::None => SidebarEntry::None,
    }
}

#[cfg(test)]
use serial_test::file_serial;
#[cfg(test)]
#[file_serial(file_fixtures)]
mod test {

    use indoc::indoc;

    use super::*;
    use crate::tests::fixtures::sidebars::SidebarFixtures;

    #[test]
    fn test_update_sidebars() {
        let sb = indoc!(
            r#"
            # Do not add comments to this file. They will be lost.

            sidebar:
              - type: section
                link: /Web/CSS
                title: CSS
              - details: closed
                title: Backgrounds_and_Borders
                children:
                  - /Web/CSS/CSS_Backgrounds_and_Borders/Using_multiple_backgrounds
                  - link: /Web/CSS/CSS_Backgrounds_and_Borders/Resizing_background_images
                    title: Resizing_background_images
              - details: closed
                title: Box alignment
                children:
                  - link: /Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Block_Abspos_Tables
                    title: Box_alignment_in_block_layout
                  - /Web/CSS/CSS_Box_Alignment/Box_Alignment_in_Flexbox
                  - /Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Grid_Layout
                  - /Web/CSS/CSS_Box_Alignment/Box_Alignment_in_Multi-column_Layout
              - details: closed
                title: Box_model
                children:
                  - /Web/CSS/CSS_Box_Model/Introduction_to_the_CSS_box_model
                  - /Web/CSS/CSS_Box_Model/Mastering_margin_collapsing
              - type: listSubPages
                path: /en-US/docs/Web/HTTP/Headers
                title: Headers
                tags: []
                details: closed
              - type: listSubPages
                path: /en-US/docs/Web/CSS/CSS_Box_Alignment
                title: Headers
                tags: []
                details: closed
              - link: /Web/CSS/CSS_Box_Alignment

            l10n:
              en-US:
                Backgrounds_and_Borders: Tutorials
                Box_alignment_in_block_layout: CSS first steps
                Box_model: Box model
              fr:
                Backgrounds_and_Borders: Tutoriels
                Box_alignment_in_block_layout: Les premiers pas de CSS
                Box_model: Box model
            "#
        );

        let _sidebars = SidebarFixtures::new(vec![sb]);
        let pairs = vec![
            (
                Cow::Borrowed("Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Block_Abspos_Tables"),
                Some(Cow::Borrowed("Web/CSS/CSS_Box_Alignment/Something_New")),
            ),
            (
                Cow::Borrowed("Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Grid_Layout"),
                Some(Cow::Borrowed("Web/CSS/CSS_Box_Alignment/Also_New")),
            ),
            (
                Cow::Borrowed("Web/HTTP/Headers"),
                Some(Cow::Borrowed("Web/HTTP/Headers_New")),
            ),
            (
                Cow::Borrowed("/Web/CSS/CSS_Box_Alignment/Box_Alignment_in_Multi-column_Layout"),
                None,
            ),
            (Cow::Borrowed("/Web/CSS/CSS_Box_Alignment"), None),
        ];
        let res = update_sidebars(&pairs);
        assert!(res.is_ok());

        // re-read the files and check if the changes are there
        let mut path = content_root().to_path_buf().to_path_buf();
        path.push("sidebars");
        path.push("sidebar_0.yaml");
        let content = fs::read_to_string(&path).unwrap();
        // tracing::info!("{}", content);
        let sb = serde_yaml_ng::from_str::<Sidebar>(&content).unwrap();

        // replacement of link of the first child in the third item of the sidebar
        let third_item_first_child =
            if let SidebarEntry::Default(BasicEntry { children, .. }) = &sb.sidebar[2] {
                children.first().unwrap()
            } else {
                panic!("Expected a Section entry with children");
            };
        let link = if let SidebarEntry::Default(BasicEntry {
            core: CoreEntry { link: l, .. },
            ..
        }) = third_item_first_child
        {
            l.clone().unwrap()
        } else {
            panic!("Expected a Link entry");
        };
        assert_eq!(link, "/Web/CSS/CSS_Box_Alignment/Something_New".to_string());

        // replacement of link of the third child in the third item of the sidebar
        let third_item_third_child =
            if let SidebarEntry::Default(BasicEntry { children, .. }) = &sb.sidebar[2] {
                children.get(2).unwrap()
            } else {
                panic!("Expected a Section entry with children");
            };
        let link = if let SidebarEntry::Link(l) = third_item_third_child {
            l.clone()
        } else {
            panic!("Expected a Link entry");
        };
        assert_eq!(link, "/Web/CSS/CSS_Box_Alignment/Also_New".to_string());

        // replacement of the path of the fifth item in the sidebar (listSubPages)
        if let SidebarEntry::ListSubPages(SubPageEntry { path, .. }) = &sb.sidebar[4] {
            assert_eq!(path, "/en-US/docs/Web/HTTP/Headers_New");
        } else {
            panic!("Expected a listSubPages entry with a path field as the fifth entry");
        };

        // last child of the third item was removed, count 4 -> 5
        let third_item_children =
            if let SidebarEntry::Default(BasicEntry { children, .. }) = &sb.sidebar[2] {
                children
            } else {
                panic!("Expected a Section entry with children on the third item");
            };
        assert_eq!(third_item_children.len(), 3);

        // second listSubPages was removed entirely
        let sixth_entry = &sb.sidebar.get(5);
        assert!(sixth_entry.is_none());
        // last default entry was removed
        let seventh_entry = &sb.sidebar.get(6);
        assert!(seventh_entry.is_none());
    }
}
