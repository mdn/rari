use std::borrow::Cow;
use std::collections::HashMap;
use std::iter::once;
use std::sync::LazyLock;

use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::error::DocError;
use crate::helpers::l10n::l10n_json_data;
use crate::helpers::subpages::get_sub_pages;
use crate::html::sidebar::{
    Details, MetaChildren, MetaSidebar, SidebarMetaEntry, SidebarMetaEntryContent,
};
use crate::pages::page::{Page, PageLike};
use crate::pages::types::doc::Doc;

static BASE: &str = "%Base%";

pub fn sidebar(slug: &str, locale: Locale) -> Result<MetaSidebar, DocError> {
    let constructor_label = l10n_json_data("Common", "Constructor", locale)?;
    let static_methods_label = l10n_json_data("Common", "Static_methods", locale)?;
    let static_properties_label = l10n_json_data("Common", "Static_properties", locale)?;
    let instance_methods_label = l10n_json_data("Common", "Instance_methods", locale)?;
    let instance_properties_label = l10n_json_data("Common", "Instance_properties", locale)?;
    let inheritance_label = l10n_json_data("Common", "Inheritance", locale)?;
    let related_labl = l10n_json_data("Common", "Related_pages_wo_group", locale)?;

    let main_object = slug_to_object_name(slug);
    let mut inheritance = vec![Cow::Borrowed(main_object.as_ref())];
    if let Some(data) = inheritance_data(&main_object) {
        inheritance.push(Cow::Borrowed(data));
    }
    if !matches!(
        main_object.as_ref(),
        "Proxy" | "Atomics" | "Math" | "Intl" | "JSON" | "Reflect" | "Temporal",
    ) {
        // %Base% is the default inheritance when the class has no extends clause:
        // instances inherit from Object.prototype, and class inherits from Function.prototype
        inheritance.push(Cow::Borrowed(BASE));
    }

    let group = get_group(&main_object, &inheritance);
    let inheritance_chain: Vec<_> = inheritance
        .iter()
        .map(|obj| JSRefItem::from_obj_str(obj, obj.as_ref() == main_object))
        .collect();

    let mut entries = vec![];

    entries.push(SidebarMetaEntry {
        section: true,
        content: SidebarMetaEntryContent::Page(Doc::page_from_slug(
            "Web/JavaScript/Reference/Global_Objects",
            locale,
            true,
        )?),
        ..Default::default()
    });

    for (index, item) in inheritance_chain.into_iter().enumerate() {
        if index == 1 {
            entries.push(SidebarMetaEntry {
                section: true,
                content: SidebarMetaEntryContent::Link {
                    title: Some(inheritance_label.to_string()),
                    link: None,
                },
                ..Default::default()
            });
        }

        entries.push(SidebarMetaEntry {
            section: true,
            code: true,
            content: SidebarMetaEntryContent::Link {
                title: Some(item.title.to_string()),
                link: Some(format!(
                    "/Web/JavaScript/Reference/Global_Objects/{}",
                    item.sub_path
                )),
            },
            ..Default::default()
        });

        let details = if item.default_opened {
            Details::Open
        } else {
            Details::Closed
        };

        for (label, list) in &[
            (constructor_label, &item.constructors),
            (static_methods_label, &item.static_methods),
            (static_properties_label, &item.static_properties),
            (instance_methods_label, &item.instance_methods),
            (instance_properties_label, &item.instance_properties),
        ] {
            let children: Vec<_> = list
                .iter()
                .map(|page| build_member_entry(page, &item.sub_pages))
                .collect();
            if !children.is_empty() {
                entries.push(SidebarMetaEntry {
                    details,
                    content: SidebarMetaEntryContent::Link {
                        title: Some(label.to_string()),
                        link: None,
                    },
                    children: MetaChildren::Children(children),
                    ..Default::default()
                })
            }
        }
    }

    if !group.is_empty() {
        entries.push(SidebarMetaEntry {
            section: true,
            content: SidebarMetaEntryContent::Link {
                title: Some(related_labl.to_string()),
                link: None,
            },
            ..Default::default()
        });
        for g in group {
            entries.push(SidebarMetaEntry {
                section: true,
                code: true,
                content: SidebarMetaEntryContent::Link {
                    title: Some(g.to_string()),
                    link: Some(format!(
                        "/Web/JavaScript/Reference/Global_Objects/{}",
                        g.replace('.', "/")
                    )),
                },
                ..Default::default()
            });
        }
    }

    Ok(MetaSidebar {
        entries,
        ..Default::default()
    })
}

#[derive(Debug, Default)]
struct JSRefItem {
    pub title: Cow<'static, str>,
    pub sub_path: Cow<'static, str>,
    pub default_opened: bool,
    pub constructors: Vec<Page>,
    pub static_methods: Vec<Page>,
    pub static_properties: Vec<Page>,
    pub instance_methods: Vec<Page>,
    pub instance_properties: Vec<Page>,
    /// Sub-pages of any depth-1 member, keyed by the parent page's slug.
    /// Used to nest e.g. `Proxy/Proxy/*` handler traps under `Proxy()`, or
    /// `Intl/Segmenter/segment/Segments` (and its own methods) under
    /// `Intl.Segmenter#segment`.
    pub sub_pages: HashMap<String, Vec<Page>>,
}

/// Recursively build a sidebar entry for a member page, nesting any sub-pages
/// (looked up by parent slug) underneath. Entries with children render as a
/// `<details>` (closed by default — unlike the surrounding member groups,
/// which open by default for the current class).
fn build_member_entry(page: &Page, sub_pages: &HashMap<String, Vec<Page>>) -> SidebarMetaEntry {
    let children: Vec<_> = sub_pages
        .get(page.slug())
        .map(|subs| {
            subs.iter()
                .map(|sub| build_member_entry(sub, sub_pages))
                .collect()
        })
        .unwrap_or_default();
    let (entry_details, meta_children) = if children.is_empty() {
        (Details::None, MetaChildren::None)
    } else {
        (Details::Closed, MetaChildren::Children(children))
    };
    SidebarMetaEntry {
        code: true,
        details: entry_details,
        content: SidebarMetaEntryContent::Link {
            title: None,
            link: page.url().strip_prefix("/en-US/docs").map(String::from),
        },
        children: meta_children,
        ..Default::default()
    }
}

fn is_prototyp_member_page(page_typ: PageType) -> bool {
    matches!(
        page_typ,
        PageType::JavascriptInstanceAccessorProperty
            | PageType::JavascriptInstanceDataProperty
            | PageType::JavascriptInstanceMethod
    )
}

impl JSRefItem {
    pub fn from_obj_str(obj: &str, open: bool) -> Self {
        let object_path = obj.replace('.', "/");
        let title = if obj == BASE {
            Cow::Borrowed("Object/Function")
        } else {
            Cow::Owned(obj.to_string())
        };
        let sub_path = if obj == BASE {
            Cow::Borrowed("Object")
        } else {
            Cow::Owned(object_path.clone())
        };
        let mut constructors = vec![];
        let mut instance_properties = vec![];
        let mut instance_methods = vec![];
        let mut static_properties = vec![];
        let mut static_methods = vec![];
        let mut sub_pages: HashMap<String, Vec<Page>> = HashMap::new();
        if obj == BASE {
            let instance_props = get_sub_pages(
                "/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object",
                Some(1),
                Default::default(),
            )
            .unwrap_or_default();
            for page in instance_props
                .into_iter()
                .filter(|page| is_prototyp_member_page(page.page_type()))
            {
                match page.page_type() {
                    PageType::JavascriptInstanceAccessorProperty
                    | PageType::JavascriptInstanceDataProperty => instance_properties.push(page),
                    PageType::JavascriptInstanceMethod => instance_methods.push(page),
                    _ => {}
                }
            }

            let static_props = get_sub_pages(
                "/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function",
                Some(1),
                Default::default(),
            );
            let static_props = static_props.unwrap_or_default();
            for page in static_props
                .into_iter()
                .filter(|page| is_prototyp_member_page(page.page_type()))
            {
                match page.page_type() {
                    PageType::JavascriptInstanceAccessorProperty
                    | PageType::JavascriptInstanceDataProperty => static_properties.push(page),
                    PageType::JavascriptInstanceMethod => static_methods.push(page),
                    _ => {}
                }
            }
        } else {
            // Fetch the whole sub-tree so we can nest pages like
            // `Proxy/Proxy/*` (handler traps under the constructor) or
            // `Intl/Segmenter/segment/Segments[/...]` (a method's return-type
            // class) under their parent entry.
            let class_slug = format!("Web/JavaScript/Reference/Global_Objects/{sub_path}");
            let pages = get_sub_pages(
                &format!("/en-US/docs/{class_slug}"),
                None,
                Default::default(),
            )
            .unwrap_or_default();

            for page in pages {
                let Some((parent_slug, _)) = page.slug().rsplit_once('/') else {
                    continue;
                };
                if parent_slug == class_slug {
                    match page.page_type() {
                        PageType::JavascriptInstanceAccessorProperty
                        | PageType::JavascriptInstanceDataProperty => {
                            instance_properties.push(page)
                        }
                        PageType::JavascriptInstanceMethod => instance_methods.push(page),
                        PageType::JavascriptStaticAccessorProperty
                        | PageType::JavascriptStaticDataProperty => static_properties.push(page),
                        PageType::JavascriptStaticMethod => static_methods.push(page),
                        PageType::JavascriptConstructor => constructors.push(page),
                        _ => {}
                    }
                } else {
                    sub_pages
                        .entry(parent_slug.to_string())
                        .or_default()
                        .push(page);
                }
            }
        }
        Self {
            title,
            sub_path,
            default_opened: open,
            constructors,
            static_methods,
            static_properties,
            instance_methods,
            instance_properties,
            sub_pages,
        }
    }
}

const ASYNC_ITERATOR: &[Cow<'static, str>] = &[Cow::Borrowed("AsyncGenerator")];
const FUNCTION: &[Cow<'static, str>] = &[
    Cow::Borrowed("AsyncFunction"),
    Cow::Borrowed("AsyncGeneratorFunction"),
    Cow::Borrowed("GeneratorFunction"),
];
const ITERATOR: &[Cow<'static, str>] = &[Cow::Borrowed("Generator")];
const TYPED_ARRAY: &[Cow<'static, str>] = &[
    Cow::Borrowed("TypedArray"),
    Cow::Borrowed("BigInt64Array"),
    Cow::Borrowed("BigUint64Array"),
    Cow::Borrowed("Float16Array"),
    Cow::Borrowed("Float32Array"),
    Cow::Borrowed("Float64Array"),
    Cow::Borrowed("Int8Array"),
    Cow::Borrowed("Int16Array"),
    Cow::Borrowed("Int32Array"),
    Cow::Borrowed("Uint8Array"),
    Cow::Borrowed("Uint8ClampedArray"),
    Cow::Borrowed("Uint16Array"),
    Cow::Borrowed("Uint32Array"),
];
const ERROR: &[Cow<'static, str>] = &[
    Cow::Borrowed("Error"),
    Cow::Borrowed("AggregateError"),
    Cow::Borrowed("EvalError"),
    Cow::Borrowed("InternalError"),
    Cow::Borrowed("RangeError"),
    Cow::Borrowed("ReferenceError"),
    Cow::Borrowed("SyntaxError"),
    Cow::Borrowed("TypeError"),
    Cow::Borrowed("URIError"),
];

static INTL_SUBPAGES: LazyLock<Vec<Cow<'static, str>>> =
    LazyLock::new(|| namespace_subpages("Intl"));
static TEMPORAL_SUBPAGES: LazyLock<Vec<Cow<'static, str>>> =
    LazyLock::new(|| namespace_subpages("Temporal"));

// Related pages
pub fn get_group(main_obj: &str, inheritance: &[Cow<'_, str>]) -> Vec<Cow<'static, str>> {
    static GROUP_DATA: LazyLock<Vec<&[Cow<'_, str>]>> =
        LazyLock::new(|| vec![ERROR, &INTL_SUBPAGES, &TEMPORAL_SUBPAGES, TYPED_ARRAY]);
    for g in GROUP_DATA.iter() {
        if g.contains(&Cow::Borrowed(main_obj)) {
            return g
                .iter()
                .filter(|x| !inheritance.contains(x))
                .map(|x| Cow::Borrowed(x.as_ref()))
                .collect();
        }
    }
    vec![]
}

fn inheritance_data(obj: &str) -> Option<&str> {
    match obj {
        o if ASYNC_ITERATOR.iter().any(|x| x == o) => Some("AsyncIterator"),
        o if FUNCTION.iter().any(|x| x == o) => Some("Function"),
        o if ITERATOR.iter().any(|x| x == o) => Some("Iterator"),
        o if TYPED_ARRAY[1..].iter().any(|x| x == o) => Some("TypedArray"),
        o if ERROR[1..].iter().any(|x| x == o) => Some("Error"),
        _ => None,
    }
}

/// Intl and Temporal are big namespaces with many classes underneath it. The classes
/// are shown as related pages.
fn namespace_subpages(namespace: &str) -> Vec<Cow<'_, str>> {
    once(Cow::Borrowed(namespace))
        .chain(
            get_sub_pages(
                &format!("/en-US/docs/Web/JavaScript/Reference/Global_Objects/{namespace}"),
                Some(1),
                Default::default(),
            )
            .unwrap_or_default()
            .iter()
            .filter(|page| {
                matches!(
                    page.page_type(),
                    PageType::JavascriptClass | PageType::JavascriptNamespace
                )
            })
            .map(|page| Cow::Owned(slug_to_object_name(page.slug()).to_string())),
        )
        .collect()
}

fn slug_to_object_name(slug: &str) -> Cow<'_, str> {
    let sub_path = slug
        .strip_prefix("Web/JavaScript/Reference/Global_Objects/")
        .unwrap_or_default();
    for namespace in &["Intl/", "Temporal/"] {
        if let Some(sub_sub_path) = sub_path.strip_prefix(namespace) {
            if sub_sub_path
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase())
                .unwrap_or_default()
            {
                return Cow::Borrowed(&namespace[..namespace.len() - 1]);
            }
            return Cow::Owned(
                sub_path[..sub_sub_path
                    .find('/')
                    .map(|i| i + namespace.len())
                    .unwrap_or(sub_path.len())]
                    .replace('/', "."),
            );
        }
    }

    sub_path[..sub_path.find('/').unwrap_or(sub_path.len())].into()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slug_to_object_name() {
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Intl/supportedValuesOf"),
            "Intl"
        );
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Intl/Collator"),
            "Intl.Collator"
        );
        assert_eq!(
            slug_to_object_name(
                "Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/format"
            ),
            "Intl.DateTimeFormat"
        );
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Temporal/Now"),
            "Temporal.Now"
        );
        assert_eq!(
            slug_to_object_name(
                "Web/JavaScript/Reference/Global_Objects/Temporal/Now/plainDateISO"
            ),
            "Temporal.Now"
        );
        assert_eq!(
            slug_to_object_name(
                "Web/JavaScript/Reference/Global_Objects/ArrayBuffer/maxByteLength"
            ),
            "ArrayBuffer"
        );
        assert_eq!(
            slug_to_object_name(
                "Web/JavaScript/Reference/Global_Objects/ArrayBuffer/maxByteLength"
            ),
            "ArrayBuffer"
        );
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Array"),
            "Array"
        );
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Proxy"),
            "Proxy"
        );
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Proxy/Proxy"),
            "Proxy"
        );
        assert_eq!(
            slug_to_object_name("Web/JavaScript/Reference/Global_Objects/Proxy/Proxy/get"),
            "Proxy"
        );
        assert_eq!(
            slug_to_object_name(
                "Web/JavaScript/Reference/Global_Objects/Intl/Segmenter/segment/Segments"
            ),
            "Intl.Segmenter"
        );
        assert_eq!(
            slug_to_object_name(
                "Web/JavaScript/Reference/Global_Objects/Intl/Segmenter/segment/Segments/containing"
            ),
            "Intl.Segmenter"
        );
    }

    #[test]
    fn test_group() {
        assert_eq!(
            get_group(
                "EvalError",
                &[
                    "EvalError".into(),
                    inheritance_data("EvalError").unwrap().into()
                ]
            ),
            vec![
                "AggregateError",
                "InternalError",
                "RangeError",
                "ReferenceError",
                "SyntaxError",
                "TypeError",
                "URIError"
            ]
        );
    }
}
