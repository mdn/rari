use std::fs;

use const_format::concatcp;
use pretty_yaml::config::{FormatOptions, LanguageOptions};
use rari_doc::html::sidebar::{BasicEntry, Sidebar, SidebarEntry, SubPageEntry, WebExtApiEntry};
use rari_types::globals::content_root;
use rari_types::locale::default_locale;
use rari_utils::concat_strs;

use crate::error::ToolError;

const PREFIX: &str = "# Do not add comments to this file. They will be lost.\n\n";
static SIDEBAR_PATH_PREFIX: &str = concatcp!("/", default_locale().as_url_str(), "/docs");

pub(crate) fn update_sidebars(pairs: &[(String, Option<String>)]) -> Result<(), ToolError> {
    let sidebars = read_sidebars()?;

    // add leading slash to pairs, because that is what the sidebars use
    let pairs = &pairs
        .iter()
        .map(|(from, to)| {
            let from = if from.starts_with('/') {
                from.to_string()
            } else {
                format!("/{}", from)
            };
            let to = if let Some(to) = to {
                if to.starts_with('/') {
                    Some(to.to_string())
                } else {
                    Some(format!("/{}", to))
                }
            } else {
                None
            };
            (from, to)
        })
        .collect::<Vec<(String, Option<String>)>>();

    // Walk the sidebars and potentially replace the links.
    // `process_entry`` is called recursively to process all children
    for mut parsed_sidebar in sidebars {
        let path = parsed_sidebar.0.clone();
        // Store a clone to detect changes later
        let original = parsed_sidebar.1.clone();
        let entries = parsed_sidebar
            .1
            .sidebar
            .into_iter()
            .map(|entry| process_entry(entry, pairs))
            .collect();

        // If the sidebar contents have changed, write it back to the file.
        if original.sidebar != entries {
            parsed_sidebar.1.sidebar = entries;

            let y = serde_yaml_ng::to_string(&parsed_sidebar.1).unwrap();
            // Format yaml a bit prettier than serde does
            let y = pretty_yaml::format_text(
                &y,
                &FormatOptions {
                    language: LanguageOptions {
                        quotes: pretty_yaml::config::Quotes::PreferDouble,
                        indent_block_sequence_in_map: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )?;
            let yaml = concat_strs!(PREFIX, &y);
            fs::write(&path, &yaml).unwrap();
        }
    }

    Ok(())
}

fn read_sidebars() -> Result<Vec<(std::path::PathBuf, Sidebar)>, ToolError> {
    // read all sidebars
    let mut path = content_root().to_path_buf();
    path.push("sidebars");
    let entries = fs::read_dir(&path).unwrap();

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

fn replace_pairs(link: Option<String>, pairs: &[(String, Option<String>)]) -> Option<String> {
    match link {
        Some(link) => {
            let mut has_prefix = false;
            let link = if let Some(l) = link.strip_prefix(SIDEBAR_PATH_PREFIX) {
                has_prefix = true;
                l.to_string()
            } else {
                link
            };
            for (from, to) in pairs {
                if link == *from {
                    if let Some(to) = to {
                        if has_prefix {
                            return Some(concat_strs!(SIDEBAR_PATH_PREFIX, to));
                        } else {
                            return Some(to.clone());
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

fn process_entry(entry: SidebarEntry, pairs: &[(String, Option<String>)]) -> SidebarEntry {
    match entry {
        SidebarEntry::Section(BasicEntry {
            link,
            hash,
            title,
            code,
            children,
            details,
        }) => {
            let new_link: Option<String> = replace_pairs(link.clone(), pairs);
            if link.is_some() && new_link.is_none() {
                return SidebarEntry::None;
            }
            SidebarEntry::Section(BasicEntry {
                link: new_link,
                hash,
                title,
                code,
                children: children
                    .into_iter()
                    .map(|c| process_entry(c, pairs))
                    .collect(),
                details,
            })
        }
        SidebarEntry::ListSubPages(SubPageEntry {
            details,
            tags,
            link,
            hash,
            title,
            path,
            include_parent,
        }) => {
            let new_path: Option<String> = replace_pairs(Some(path), pairs);
            if new_path.is_none() {
                return SidebarEntry::None;
            }
            SidebarEntry::ListSubPages(SubPageEntry {
                details,
                tags,
                link: replace_pairs(link.clone(), pairs),
                hash,
                title,
                path: new_path.unwrap(),
                include_parent,
            })
        }
        SidebarEntry::ListSubPagesGrouped(SubPageEntry {
            details,
            tags,
            link,
            hash,
            title,
            path,
            include_parent,
        }) => {
            let new_path: Option<String> = replace_pairs(Some(path), pairs);
            if new_path.is_none() {
                return SidebarEntry::None;
            }
            SidebarEntry::ListSubPagesGrouped(SubPageEntry {
                details,
                tags,
                link: replace_pairs(link.clone(), pairs),
                hash,
                title,
                path: new_path.unwrap(),
                include_parent,
            })
        }
        SidebarEntry::Default(BasicEntry {
            link,
            hash,
            title,
            code,
            children,
            details,
        }) => SidebarEntry::Default(BasicEntry {
            link: replace_pairs(link.clone(), pairs),
            hash,
            title,
            code,
            children: children
                .into_iter()
                .map(|c| process_entry(c, pairs))
                .collect(),
            details,
        }),
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
                "Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Block_Abspos_Tables".to_string(),
                Some("Web/CSS/CSS_Box_Alignment/Something_New".to_string()),
            ),
            (
                "Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Grid_Layout".to_string(),
                Some("Web/CSS/CSS_Box_Alignment/Also_New".to_string()),
            ),
            (
                "Web/HTTP/Headers".to_string(),
                Some("Web/HTTP/Headers_New".to_string()),
            ),
            (
                "/Web/CSS/CSS_Box_Alignment/Box_Alignment_in_Multi-column_Layout".to_string(),
                None,
            ),
            ("/Web/CSS/CSS_Box_Alignment".to_string(), None),
        ];
        let res = update_sidebars(&pairs);
        assert!(res.is_ok());

        // re-read the files and check if the changes are there
        let mut path = content_root().to_path_buf().to_path_buf();
        path.push("sidebars");
        path.push("sidebar_0.yaml");
        let content = fs::read_to_string(&path).unwrap();
        // println!("{}", content);
        let sb = serde_yaml_ng::from_str::<Sidebar>(&content).unwrap();

        // replacement of link of the first child in the third item of the sidebar
        let third_item_first_child =
            if let SidebarEntry::Default(BasicEntry { children, .. }) = &sb.sidebar[2] {
                children.first().unwrap()
            } else {
                panic!("Expected a Section entry with children");
            };
        let link = if let SidebarEntry::Default(BasicEntry { link: l, .. }) = third_item_first_child
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
    }
}
