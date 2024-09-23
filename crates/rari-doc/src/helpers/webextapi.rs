use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use super::l10n::l10n_json_data;
use crate::error::DocError;
use crate::helpers::subpages::{get_sub_pages, SubPagesSorter};
use crate::helpers::titles::api_page_title;
use crate::html::sidebar::{Details, MetaChildren, SidebarMetaEntry, SidebarMetaEntryContent};
use crate::pages::page::{Page, PageLike};

pub fn entry(slug: &str, locale: Locale) -> Result<Vec<SidebarMetaEntry>, DocError> {
    let properties_label = l10n_json_data("Common", "Properties", locale)?;
    let methods_label = l10n_json_data("Common", "Methods", locale)?;
    let types_label = l10n_json_data("Common", "Types", locale)?;
    let events_label = l10n_json_data("Common", "Events", locale)?;

    let sub_pages = get_sub_pages(
        &format!("/en-US/docs/{}", slug),
        Some(1),
        SubPagesSorter::TitleAPI,
    )?;

    let mut properties = vec![];
    let mut methods = vec![];
    let mut types = vec![];
    let mut events = vec![];

    for page in sub_pages {
        let v = match page.page_type() {
            PageType::WebextensionApiProperty => &mut properties,
            PageType::WebextensionApiFunction => &mut methods,
            PageType::WebextensionApiType => &mut types,
            PageType::WebextensionApiEvent => &mut events,
            _ => continue,
        };
        v.push(page);
    }

    let mut children = vec![];
    build_sublist(&mut children, &methods, methods_label);
    build_sublist(&mut children, &properties, properties_label);
    build_sublist(&mut children, &types, types_label);
    build_sublist(&mut children, &events, events_label);
    Ok(children)
}

pub fn children(slug: &str, locale: Locale) -> Result<Vec<SidebarMetaEntry>, DocError> {
    let sub_pages = get_sub_pages(
        "/en-US/docs/Mozilla/Add-ons/WebExtensions/API",
        Some(1),
        SubPagesSorter::TitleAPI,
    )?;

    let mut entries = vec![];

    build_sublist_and_expand_current(
        &mut entries,
        &sub_pages,
        "/Mozilla/Add-ons/WebExtensions/Browser_support_for_JavaScript_APIs",
        slug,
        locale,
    )?;

    Ok(entries)
}

fn build_sublist(entries: &mut Vec<SidebarMetaEntry>, sub_pages: &[Page], label: &str) {
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

fn build_sublist_and_expand_current(
    entries: &mut Vec<SidebarMetaEntry>,
    sub_pages: &[Page],
    link: &str,
    current_slug: &str,
    locale: Locale,
) -> Result<(), DocError> {
    if !sub_pages.is_empty() {
        entries.push(SidebarMetaEntry {
            content: SidebarMetaEntryContent::Link {
                link: Some(link.to_string()),
                title: None,
            },
            ..Default::default()
        });
        entries.extend(
            sub_pages
                .iter()
                .map(|page| -> Result<SidebarMetaEntry, _> {
                    Ok::<_, DocError>(SidebarMetaEntry {
                        content: SidebarMetaEntryContent::Link {
                            title: Some(api_page_title(page).to_string()),
                            link: page
                                .clone()
                                .url()
                                .strip_prefix("/en-US/docs")
                                .map(String::from),
                        },
                        children: if page.slug() == current_slug {
                            MetaChildren::Children(entry(current_slug, locale)?)
                        } else {
                            Default::default()
                        },
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    Ok(())
}
