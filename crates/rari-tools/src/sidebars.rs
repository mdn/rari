use std::fs;

use rari_doc::html::sidebar::{BasicEntry, Sidebar, SidebarEntry, SubPageEntry, WebExtApiEntry};
use rari_types::globals::content_root;

use crate::error::ToolError;

pub(crate) fn update_sidebars(pairs: &[(String, String)]) -> Result<(), ToolError> {
    // read all sidebars
    let mut path = content_root().to_path_buf();
    path.push("sidebars");
    let entries = fs::read_dir(&path).unwrap();

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
            println!("content: {}", content);
            let sidebar: Sidebar = serde_yaml_ng::from_str(&content).unwrap();
            (path, sidebar)
        })
        .collect::<Vec<(std::path::PathBuf, Sidebar)>>();

    for mut parsed_sidebar in sidebars {
        let path = parsed_sidebar.0.clone();
        let entries = parsed_sidebar
            .1
            .sidebar
            .into_iter()
            .map(|entry| process_entry(entry, pairs))
            .collect();
        parsed_sidebar.1.sidebar = entries;

        let y = serde_yaml_ng::to_string(&parsed_sidebar.1).unwrap();
        println!("sidebar {}: \n{}", path.to_string_lossy(), y);
    }

    Ok(())
}

fn process_entry(entry: SidebarEntry, _pairs: &[(String, String)]) -> SidebarEntry {
    match entry {
        SidebarEntry::Section(BasicEntry {
            link,
            title,
            code,
            children,
            details,
        }) => SidebarEntry::Section(BasicEntry {
            link,
            title,
            code,
            children,
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
            link,
            title,
            path,
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
            link,
            title,
            path,
            include_parent,
        }),
        SidebarEntry::Default(BasicEntry {
            link: _link,
            title,
            code,
            children,
            details,
        }) => SidebarEntry::Default(BasicEntry {
            link: _link, //Some("Hey".to_string()),
            title,
            code,
            children,
            details,
        }),
        SidebarEntry::Link(link) => SidebarEntry::Link(link),
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
    }
}
