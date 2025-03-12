use std::borrow::Cow;

use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::api_inheritance::inheritance;
use crate::helpers::json_data::json_data_group;
use crate::helpers::l10n::l10n_json_data;
use crate::helpers::subpages::{get_sub_pages, SubPagesSorter};
use crate::helpers::titles::api_page_title;
use crate::html::sidebar::{
    Details, MetaChildren, MetaSidebar, SidebarMetaEntry, SidebarMetaEntryContent,
};
use crate::pages::page::{Page, PageLike};
use crate::pages::types::doc::Doc;

pub fn sidebar(slug: &str, group: Option<&str>, locale: Locale) -> Result<MetaSidebar, DocError> {
    let static_properties_label = l10n_json_data("Common", "Static_properties", locale)?;
    let instance_properties_label = l10n_json_data("Common", "Instance_properties", locale)?;
    let static_methods_label = l10n_json_data("Common", "Static_methods", locale)?;
    let instance_methods_label = l10n_json_data("Common", "Instance_methods", locale)?;
    let constructor_label = l10n_json_data("Common", "Constructor", locale)?;
    let inheritance_label = l10n_json_data("Common", "Inheritance", locale)?;
    let related_label = if let Some(group) = group {
        Cow::Owned(l10n_json_data("Common", "Related_pages", locale)?.replace("$1", group))
    } else {
        Cow::Borrowed(l10n_json_data("Common", "Related_pages_wo_group", locale)?)
    };
    let events_label = l10n_json_data("Common", "Events", locale)?;

    let main_if = slug
        .strip_prefix("Web/API/")
        .map(|s| &s[..s.find('/').unwrap_or(s.len())])
        .ok_or_else(|| DocError::InvalidSlugForX(slug.to_string()))?;

    let web_api_groups = group.and_then(|group| json_data_group().get(group));

    let main_if_pages = get_sub_pages(
        &format!("/en-US/docs/Web/API/{}", main_if),
        Some(1),
        SubPagesSorter::TitleAPI,
    )?;

    let related = if let Some(iter) = web_api_groups.map(|groups| {
        groups
            .interfaces
            .iter()
            .chain(groups.methods.iter())
            .chain(groups.properties.iter())
            .filter(|s| s.as_str() != main_if)
            .map(|s| s.as_str())
    }) {
        let mut related = iter.collect::<Vec<_>>();
        related.sort();
        related
    } else {
        Default::default()
    };

    let mut static_properties = vec![];
    let mut instance_properties = vec![];
    let mut static_methods = vec![];
    let mut instance_methods = vec![];
    let mut constructors = vec![];
    let mut events = vec![];

    for page in main_if_pages {
        let v = match page.page_type() {
            PageType::WebApiStaticProperty => &mut static_properties,
            PageType::WebApiInstanceProperty => &mut instance_properties,
            PageType::WebApiStaticMethod => &mut static_methods,
            PageType::WebApiInstanceMethod => &mut instance_methods,
            PageType::WebApiConstructor => &mut constructors,
            PageType::WebApiEvent => &mut events,
            _ => continue,
        };
        v.push(page);
    }

    let inherited = inheritance(main_if);

    let mut entries = vec![];

    if let Some([ref overview, ..]) = web_api_groups.map(|group| group.overview.as_slice()) {
        entries.push(SidebarMetaEntry {
            section: true,
            content: SidebarMetaEntryContent::Page(Doc::page_from_slug(
                &format!("Web/API/{}", overview.replace(' ', "_")),
                locale,
                true,
            )?),
            ..Default::default()
        });
    }
    entries.push(SidebarMetaEntry {
        section: true,
        code: true,
        content: SidebarMetaEntryContent::Page(Doc::page_from_slug(
            &format!("Web/API/{main_if}"),
            locale,
            true,
        )?),
        ..Default::default()
    });

    build_sublist(&mut entries, &constructors, constructor_label);
    build_sublist(&mut entries, &static_properties, static_properties_label);
    build_sublist(
        &mut entries,
        &instance_properties,
        instance_properties_label,
    );
    build_sublist(&mut entries, &static_methods, static_methods_label);
    build_sublist(&mut entries, &instance_methods, instance_methods_label);
    build_sublist(&mut entries, &events, events_label);

    build_interface_list(&mut entries, &inherited, inheritance_label);
    build_interface_list(&mut entries, &related, &related_label);

    Ok(MetaSidebar {
        entries,
        ..Default::default()
    })
}

fn build_sublist(entries: &mut Vec<SidebarMetaEntry>, sub_pages: &[Page], label: &str) {
    if !sub_pages.is_empty() {
        entries.push(SidebarMetaEntry {
            details: Details::Open,
            content: SidebarMetaEntryContent::Link {
                link: None,
                title: Some(label.replace("_static", "").to_string()),
            },
            children: MetaChildren::Children(
                sub_pages
                    .iter()
                    .map(|page| SidebarMetaEntry {
                        code: true,
                        content: SidebarMetaEntryContent::Link {
                            title: Some(api_page_title(page).to_string()),
                            link: page
                                .clone()
                                .url()
                                .strip_prefix("/en-US/docs")
                                .map(String::from),
                        },
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        })
    }
}

fn build_interface_list(entries: &mut Vec<SidebarMetaEntry>, interfaces: &[&str], label: &str) {
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
                    .map(|interface| SidebarMetaEntry {
                        code: true,
                        content: SidebarMetaEntryContent::Link {
                            title: Some(interface.replace("_static", "").to_string()),
                            link: Some(format!(
                                "/Web/API/{}",
                                interface.replace("()", "").replace('.', "/")
                            )),
                        },
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        })
    }
}
