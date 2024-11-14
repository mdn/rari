use std::{fs, iter};

use rari_doc::html::sidebar::{BasicEntry, Sidebar, SidebarEntry, SubPageEntry, WebExtApiEntry};
use rari_types::globals::content_root;
use rari_utils::concat_strs;

use crate::error::ToolError;

pub(crate) fn update_sidebars(pairs: &[(String, String)]) -> Result<(), ToolError> {
    // read all sidebars
    let mut path = content_root().to_path_buf();
    path.push("sidebars");
    let entries = fs::read_dir(&path).unwrap();

    // map and parse sidebars into a vector of (path, Sidebar)
    let sidebars = entries
        .filter(|entry| {
            let entry = entry.as_ref().unwrap();
            let path = entry.path();
            path.is_file() && path.extension().unwrap() == "yaml"
        })
        .map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            let content = fs::read_to_string(&path).unwrap();
            println!("input content {}: \n\n{}", path.to_string_lossy(), content);
            let sidebar: Sidebar = serde_yaml_ng::from_str(&content).unwrap();
            (path, sidebar)
        })
        .collect::<Vec<(std::path::PathBuf, Sidebar)>>();

    // add leading slash to pairs, because that is what the sidebars use
    let pairs = &pairs
        .iter()
        .map(|(from, to)| {
            let from = if from.starts_with('/') {
                from.to_string()
            } else {
                format!("/{}", from)
            };
            let to = if to.starts_with('/') {
                to.to_string()
            } else {
                format!("/{}", to)
            };
            (from, to)
        })
        .collect::<Vec<(String, String)>>();

    // walk the sidebars and potentially replace the links
    // process_entry is called recursively to process all children
    for mut parsed_sidebar in sidebars {
        let path = parsed_sidebar.0.clone();
        let entries = parsed_sidebar
            .1
            .sidebar
            .into_iter()
            .map(|entry| process_entry(entry, pairs))
            .collect();

        // TODO: write the modified data back to the sidebar file
        parsed_sidebar.1.sidebar = entries;
        const PREFIX: &str = "# Do not add comments to this file. They will be lost.\n\n";
        let y = serde_yaml_ng::to_string(&parsed_sidebar.1).unwrap();
        let yaml = concat_strs!(PREFIX, &y);
        println!("output content {}: \n\n{}", path.to_string_lossy(), yaml);
    }

    Ok(())
}

fn replace_pairs(pairs: &[(String, String)]) -> impl FnMut(Option<String>) -> Option<String> + '_ {
    move |link: Option<String>| match link {
        Some(link) => {
            for (from, to) in pairs {
                if link == *from {
                    return Some(to.clone());
                }
            }
            Some(link)
        }
        None => None,
    }
}

fn process_entry(entry: SidebarEntry, pairs: &[(String, String)]) -> SidebarEntry {
    match entry {
        SidebarEntry::Section(BasicEntry {
            link,
            title,
            code,
            children,
            details,
        }) => SidebarEntry::Section(BasicEntry {
            link: iter::once(link).map(replace_pairs(pairs)).collect(),
            title,
            code,
            children: children
                .into_iter()
                .map(|c| process_entry(c, pairs))
                .collect(),
            details,
        }),
        SidebarEntry::ListSubPages(SubPageEntry {
            details,
            tags,
            link,
            title,
            path,
            include_parent,
        }) => SidebarEntry::ListSubPages(SubPageEntry {
            details,
            tags,
            link: iter::once(link).map(replace_pairs(pairs)).collect(),
            title,
            path: iter::once(Some(path))
                .map(replace_pairs(pairs))
                .collect::<Option<String>>()
                .unwrap(),
            include_parent,
        }),
        SidebarEntry::ListSubPagesGrouped(SubPageEntry {
            details,
            tags,
            link,
            title,
            path,
            include_parent,
        }) => SidebarEntry::ListSubPagesGrouped(SubPageEntry {
            details,
            tags,
            link: iter::once(link).map(replace_pairs(pairs)).collect(),
            title,
            path: iter::once(Some(path))
                .map(replace_pairs(pairs))
                .collect::<Option<String>>()
                .unwrap(),
            include_parent,
        }),
        SidebarEntry::Default(BasicEntry {
            link,
            title,
            code,
            children,
            details,
        }) => SidebarEntry::Default(BasicEntry {
            link: iter::once(link).map(replace_pairs(pairs)).collect(),
            title,
            code,
            children: children
                .into_iter()
                .map(|c| process_entry(c, pairs))
                .collect(),
            details,
        }),
        SidebarEntry::Link(link) => SidebarEntry::Link(
            iter::once(Some(link))
                .map(replace_pairs(pairs))
                .collect::<Option<String>>()
                .unwrap(),
        ),
        SidebarEntry::WebExtApi(WebExtApiEntry { title }) => {
            SidebarEntry::WebExtApi(WebExtApiEntry { title })
        }
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
                "Web/CSS/CSS_Box_Alignment/Something_New".to_string(),
            ),
            (
                "Web/CSS/CSS_Box_Alignment/Box_Alignment_In_Grid_Layout".to_string(),
                "Web/CSS/CSS_Box_Alignment/Also_New".to_string(),
            ),
        ];
        let res = update_sidebars(&pairs);
        assert!(res.is_ok());
        // re-read the files and check if the changes are there
    }
}
