use std::borrow::Cow;
use std::iter::once;

use once_cell::sync::Lazy;
use rari_types::fm_types::PageType;
use rari_types::locale::Locale;

use crate::docs::doc::Doc;
use crate::docs::page::{Page, PageLike};
use crate::error::DocError;
use crate::html::sidebar::{
    Details, MetaChildren, MetaSidebar, SidebarMetaEntry, SidebarMetaEntryContent,
};
use crate::templ::api::RariApi;

static BASE: &str = "%Base%";

pub fn sidebar(slug: &str, locale: Locale) -> Result<MetaSidebar, DocError> {
    let main_object = slug_to_object_name(slug);
    let mut inheritance = vec![Cow::Borrowed(main_object.as_ref())];
    if let Some(data) = inheritance_data(&main_object) {
        inheritance.push(Cow::Borrowed(data));
    }
    if !matches!(
        main_object.as_ref(),
        "Proxy" | "Atomics" | "Math" | "Intl" | "JSON" | "Reflect",
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
        )?),
        ..Default::default()
    });

    for (index, item) in inheritance_chain.into_iter().enumerate() {
        if index == 1 {
            entries.push(SidebarMetaEntry {
                section: true,
                content: SidebarMetaEntryContent::Link {
                    title: Some("Inheritence".into()),
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
            ("Constructor", item.constructors),
            ("Static methods", item.static_methods),
            ("Static properties", item.static_properties),
            ("Instance methods", item.instance_methods),
            ("Instance Properties", item.instance_properties),
        ] {
            let children: Vec<_> = list
                .iter()
                .map(|page| SidebarMetaEntry {
                    code: true,
                    content: SidebarMetaEntryContent::Page(page.clone()),
                    ..Default::default()
                })
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
                title: Some("Related".into()),
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

    Ok(MetaSidebar { entries })
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
        let title = match obj {
            "Intl/Segmenter/segment/Segments" => Cow::Borrowed("Segments"),
            base if base == BASE => Cow::Borrowed("Object/Function"),
            _ => Cow::Owned(obj.to_string()),
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
        if obj == BASE {
            let instance_props = RariApi::get_sub_pages(
                "/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object",
                Some(1),
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

            let static_props = RariApi::get_sub_pages(
                "/en-US/docs/Web/JavaScript/Reference/Global_Objects/Function",
                Some(1),
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
            let pages = RariApi::get_sub_pages(
                &format!("/en-US/docs/Web/JavaScript/Reference/Global_Objects/{sub_path}"),
                Some(1),
            )
            .unwrap_or_default();

            for page in pages {
                match page.page_type() {
                    PageType::JavascriptInstanceAccessorProperty
                    | PageType::JavascriptInstanceDataProperty => instance_properties.push(page),
                    PageType::JavascriptInstanceMethod => instance_methods.push(page),
                    PageType::JavascriptStaticAccessorProperty
                    | PageType::JavascriptStaticDataProperty => static_properties.push(page),
                    PageType::JavascriptStaticMethod => static_methods.push(page),
                    PageType::JavascriptConstructor => constructors.push(page),
                    _ => {}
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
        }
    }
}

const ASYNC_GENERATOR: &[Cow<'static, str>] = &[Cow::Borrowed("AsyncGenerator")];
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

// Related pages
fn get_group(main_obj: &str, inheritance: &[Cow<'_, str>]) -> Vec<Cow<'static, str>> {
    static GROUP_DATA: Lazy<Vec<&[Cow<'static, str>]>> = Lazy::new(|| {
        vec![
            ERROR,
            &INTL_SUBPAGES,
            &[
                Cow::Borrowed("Intl/Segmenter/segment/Segments"),
                Cow::Borrowed("Intl.Segmenter"),
            ],
            TYPED_ARRAY,
            &[Cow::Borrowed("Proxy"), Cow::Borrowed("Proxy/handler")],
        ]
    });
    for g in GROUP_DATA.iter() {
        if g.iter().any(|x| main_obj == x) {
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
        o if ASYNC_GENERATOR.iter().any(|x| x == o) => Some("AsyncIterator"),
        o if FUNCTION.iter().any(|x| x == o) => Some("Function"),
        o if ITERATOR.iter().any(|x| x == o) => Some("Iterator"),
        o if TYPED_ARRAY[1..].iter().any(|x| x == o) => Some("TypedArray"),
        o if ERROR[1..].iter().any(|x| x == o) => Some("Error"),
        _ => None,
    }
}

static INTL_SUBPAGES: Lazy<Vec<Cow<'static, str>>> = Lazy::new(|| {
    once(Cow::Borrowed("Intl"))
        .chain(
            RariApi::get_sub_pages(
                "/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl",
                Some(1),
            )
            .unwrap_or_default()
            .iter()
            .filter(|page| page.page_type() == PageType::JavascriptClass)
            .map(|page| Cow::Owned(slug_to_object_name(page.slug()).to_string())),
        )
        .collect()
});

fn slug_to_object_name(slug: &str) -> Cow<'_, str> {
    let sub_path = slug
        .strip_prefix("Web/JavaScript/Reference/Global_Objects/")
        .unwrap_or_default();
    if sub_path.starts_with("Intl/Segmenter/segment/Segments") {
        return "Intl/Segmenter/segment/Segments".into();
    }
    if sub_path.starts_with("Proxy/Proxy") {
        return "Proxy/handler".into();
    }
    if let Some(intl) = sub_path.strip_prefix("Intl/") {
        if intl
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or_default()
        {
            return "Intl".into();
        }
        return Cow::Owned(
            sub_path[..intl.find('/').map(|i| i + 5).unwrap_or(sub_path.len())].replace('/', "."),
        );
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
