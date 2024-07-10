use rari_l10n::l10n_json_data;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;
use tracing::error;

use crate::docs::doc::Doc;
use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::helpers::json_data::{json_data_group, json_data_interface};
use crate::helpers::subpages::get_sub_pages;
use crate::html::sidebar::{MetaSidebar, SidebarEntry, SidebarMetaEntry, SidebarMetaEntryContent};

pub fn sidebar(slug: &str, group: Option<&str>, locale: Locale) -> Result<MetaSidebar, DocError> {
    let static_properties_label = l10n_json_data("Common", "Static_properties", locale)?;
    let instance_properties_label = l10n_json_data("Common", "Instance_properties", locale)?;
    let static_methods_label = l10n_json_data("Common", "Static_methods", locale)?;
    let instance_methods_label = l10n_json_data("Common", "Instance_methods", locale)?;
    let constructor_label = l10n_json_data("Common", "Constructor", locale)?;
    let inheritance_label = l10n_json_data("Common", "Inheritance", locale)?;
    let implemented_by_label = l10n_json_data("Common", "Implemented_by", locale)?;
    let related_labl = l10n_json_data("Common", "Related_pages_wo_group", locale)?;
    let translate_label = l10n_json_data("Common", "[Translate]", locale)?;
    let title_label = l10n_json_data("Common", "TranslationCTA", locale)?;
    let events_label = l10n_json_data("Common", "Events", locale)?;

    let main_if = slug
        .strip_prefix("Web/API/")
        .map(|s| &s[..s.find('/').unwrap_or(s.len())])
        .ok_or_else(|| DocError::InvalidSlugForX(slug.to_string()))?;
    if !main_if
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or_default()
    {
        error!("Slugs for Web/API/* must start with uppercase letter got {slug}");
        return Err(DocError::InvalidSlugForX(slug.to_string()));
    }

    let web_api_data = json_data_interface();
    let web_api_groups = group.and_then(|group| json_data_group().get(group));

    // TODO: custom sorting?
    let main_if_pages = get_sub_pages(
        &format!("/en-US/docs/Web/API/{}", main_if),
        Some(1),
        Default::default(),
    )?;

    let related = if let Some(iter) = web_api_groups.map(|groups| {
        groups
            .interfaces
            .iter()
            .chain(groups.methods.iter())
            .chain(groups.properties.iter())
            .filter(|s| s.as_str() != main_if)
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

    let mut inherited = vec![];

    let mut interface = main_if;
    while let Some(inherited_data) = web_api_data.get(interface).map(|data| data.inh.as_str()) {
        inherited.push(inherited_data);
        interface = inherited_data;
    }

    let mut entries = vec![];

    if let Some([ref overview]) = web_api_groups.map(|group| group.overview.as_slice()) {
        entries.push(SidebarMetaEntry {
            section: true,
            content: SidebarMetaEntryContent::Page(Doc::page_from_slug(
                &format!("Web/API/{}", overview.replace(' ', "_")),
                locale,
            )?),
            ..Default::default()
        });
    }
    entries.push(SidebarMetaEntry {
        section: true,
        content: SidebarMetaEntryContent::Page(Doc::page_from_slug(
            &format!("Web/API/{main_if}"),
            locale,
        )?),
        ..Default::default()
    });

    Ok(MetaSidebar { entries })
}

fn api_page_title(page: &Page) -> &str {
    if let Some(short_title) = page.short_title() {
        return short_title;
    }
    let title = page.title();
    let title = &title[title.rfind('.').unwrap_or(0)..];
    if matches!(page.page_type(), PageType::WebApiEvent) {
        let title = page.slug();
        let title = &title[title.rfind('/').unwrap_or(0)..];
        if let Some(title) = title.strip_suffix("_event") {
            title
        } else {
            title
        }
    } else {
        title
    }
}
