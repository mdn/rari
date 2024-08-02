use rari_l10n::l10n_json_data;
use rari_types::locale::Locale;

use crate::docs::doc::Doc;
use crate::error::DocError;
use crate::helpers::json_data::json_data_group;
use crate::html::sidebar::{
    Details, MetaChildren, MetaSidebar, SidebarMetaEntry, SidebarMetaEntryContent,
};

pub fn sidebar(group: &str, locale: Locale) -> Result<MetaSidebar, DocError> {
    let properties_label = l10n_json_data("Common", "Properties", locale)?;
    let methods_label = l10n_json_data("Common", "Methods", locale)?;
    let events_label = l10n_json_data("Common", "Events", locale)?;
    let interfaces_label = l10n_json_data("Common", "Interfaces", locale)?;
    let guides_label = l10n_json_data("Common", "Guides", locale)?;
    let tutorial_label = l10n_json_data("Common", "Tutorial", locale)?;

    let web_api_groups = json_data_group()
        .get(group)
        .ok_or(DocError::InvalidSlugForX(group.to_string()))?;

    let mut entries = vec![];

    if let [ref overview, ..] = web_api_groups.overview.as_slice() {
        entries.push(SidebarMetaEntry {
            section: true,
            content: SidebarMetaEntryContent::Page(Doc::page_from_slug(
                &format!("Web/API/{}", overview.replace(' ', "_")),
                locale,
            )?),
            ..Default::default()
        });
    }

    build_sublist(&mut entries, &web_api_groups.guides, guides_label);
    build_sublist(&mut entries, &web_api_groups.tutorial, tutorial_label);

    build_interface_list(
        &mut entries,
        &web_api_groups.interfaces,
        interfaces_label,
        APILink::from,
    );
    build_interface_list(
        &mut entries,
        &web_api_groups.properties,
        properties_label,
        APILink::from,
    );
    build_interface_list(
        &mut entries,
        &web_api_groups.methods,
        methods_label,
        APILink::from,
    );
    build_interface_list(
        &mut entries,
        &web_api_groups.events,
        events_label,
        APILink::from_event,
    );

    Ok(MetaSidebar {
        entries,
        ..Default::default()
    })
}

struct APILink {
    title: Option<String>,
    link: String,
}

impl APILink {
    pub fn from(s: &str) -> Option<Self> {
        Some(Self {
            title: Some(s.to_string()),
            link: format!("/Web/API/{}", s.replace("()", "").replace('.', "/")),
        })
    }

    pub fn from_event(ev: &str) -> Option<Self> {
        ev.split_once(": ").map(|(interface, event)| Self {
            link: format!("/Web/API/{interface}/{event}_event"),
            title: Some(ev.to_string()),
        })
    }
}

fn build_sublist(entries: &mut Vec<SidebarMetaEntry>, sub_pages: &[String], label: &str) {
    if !sub_pages.is_empty() {
        entries.push(SidebarMetaEntry {
            details: Details::Open,
            content: SidebarMetaEntryContent::Link {
                link: None,
                title: Some(label.to_string()),
            },
            children: MetaChildren::Children(
                sub_pages
                    .iter()
                    .map(|url| SidebarMetaEntry {
                        content: SidebarMetaEntryContent::Link {
                            title: None,
                            link: Some(url.strip_prefix("/docs").unwrap_or(url).to_string()),
                        },
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        })
    }
}

fn build_interface_list(
    entries: &mut Vec<SidebarMetaEntry>,
    interfaces: &[String],
    label: &str,
    convert: fn(&str) -> Option<APILink>,
) {
    if !interfaces.is_empty() {
        entries.push(SidebarMetaEntry {
            details: Details::Open,
            content: SidebarMetaEntryContent::Link {
                link: None,
                title: Some(label.to_string()),
            },
            children: MetaChildren::Children(
                interfaces
                    .iter()
                    .filter_map(|s| convert(s))
                    .map(|APILink { link, title }| SidebarMetaEntry {
                        code: true,
                        content: SidebarMetaEntryContent::Link {
                            title,
                            link: Some(link),
                        },
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        })
    }
}
