pub mod api_list_alpha;
pub mod api_list_specs;
pub mod badges;
pub mod banners;
pub mod compat;
pub mod css_ref;
pub mod cssinfo;
pub mod csssyntax;
pub mod echo;
pub mod embeds;
pub mod firefox_for_developers;
pub mod glossary;
pub mod glossarydisambiguation;
pub mod inheritance_diagram;
pub mod inline_labels;
pub mod js_property_attributes;
pub mod links;
pub mod list_subpages_for_sidebar;
pub mod listsubpages;
pub mod previous_menu_next;
pub mod quick_links_with_subpages;
pub mod sidebars;
pub mod specification;
pub mod subpages_with_summaries;
pub mod svginfo;
pub mod web_ext_examples;
pub mod webext_all_examples;
pub mod xsltref;

use std::collections::HashMap;
use std::sync::LazyLock;

use rari_types::globals::deny_warnings;
use rari_types::templ::{RariFn, TemplType};
use rari_types::{Arg, RariEnv};
use tracing::error;

use crate::error::DocError;
use crate::utils::{TEMPL_RECORDER, TemplStatEvent, is_unrooted};

#[derive(Debug)]
pub struct Templ {
    pub name: &'static str,
    pub outline: &'static str,
    pub outline_snippet: &'static str,
    pub outline_plain: &'static str,
    pub doc: &'static str,
    pub function: RariFn<Result<String, DocError>>,
    pub typ: TemplType,
}

inventory::collect!(Templ);

pub static TEMPL_MAP: LazyLock<Vec<&'static Templ>> =
    LazyLock::new(|| inventory::iter::<Templ>().collect());

pub static TEMPL_MAPPING: LazyLock<HashMap<&'static str, &'static Templ>> =
    LazyLock::new(|| inventory::iter::<Templ>().map(|t| (t.name, t)).collect());

pub fn exists(name: &str) -> bool {
    TEMPL_MAPPING.contains_key(name)
}

fn record_invocation(name: String, locale: rari_types::locale::Locale, known: bool) {
    TEMPL_RECORDER.with(|tx| {
        if let Some(tx) = tx
            && let Err(e) = tx.send(TemplStatEvent::Record {
                name,
                locale,
                known,
            })
        {
            error!("templ recorder: {e}");
        }
    });
}

pub fn invoke(
    env: &RariEnv,
    name: &str,
    args: Vec<Option<Arg>>,
) -> Result<(String, TemplType), DocError> {
    let name = name.replace('-', "_");
    let (f, is_sidebar) = match TEMPL_MAPPING.get(name.as_str()) {
        Some(t) => (t.function, t.typ),
        None if name == "xulelem" => return Ok((Default::default(), TemplType::None)),
        None if deny_warnings() => return Err(DocError::UnknownMacro(name.to_string())),
        None => {
            let rendered = format!("<s>unsupported templ: {name}</s>");
            record_invocation(name, env.locale, false);
            return Ok((rendered, TemplType::None));
        } //
    };
    record_invocation(name, env.locale, true);
    if matches!(is_sidebar, TemplType::Sidebar) && is_unrooted(env.slug) {
        return Ok((Default::default(), TemplType::Sidebar));
    }
    f(env, args).map(|s| (s, is_sidebar))
}

#[cfg(test)]
mod test {
    use rari_types::Quotes;
    use rari_types::fm_types::PageType;
    use rari_types::locale::Locale;

    use super::*;

    #[test]
    fn test_kw() {
        println!("{:?}", *TEMPL_MAP);
    }

    fn env_for_slug(slug: &str) -> RariEnv<'_> {
        RariEnv {
            url: "",
            locale: Locale::EnUs,
            title: "",
            tags: &[],
            browser_compat: &[],
            spec_urls: &[],
            page_type: PageType::default(),
            slug,
            status: &[],
        }
    }

    #[test]
    fn test_invoke_skips_sidebars_on_unrooted_pages() {
        let cases = vec![
            (
                "jsref",
                "conflicting/Web/JavaScript/Reference/Global_Objects/Array/toString",
            ),
            ("apiref", "orphaned/Web/API/Window/dialogArguments"),
            ("cssref", "conflicting/Web/CSS/counter-reset"),
        ];
        for (name, slug) in cases {
            let env = env_for_slug(slug);
            let (rendered, typ) = invoke(&env, name, vec![]).expect("invoke succeeds");
            assert!(
                rendered.is_empty(),
                "{name} on {slug} rendered {rendered:?}"
            );
            assert!(
                matches!(typ, TemplType::Sidebar),
                "{name} should stay a sidebar templ"
            );
        }
    }

    #[test]
    fn test_invoke_renders_non_sidebars_on_unrooted_pages() {
        let env = env_for_slug("conflicting/Web/API/Window/showModalDialog");
        let args = vec![Some(Arg::String("hi".to_string(), Quotes::Double))];
        let (rendered, _) = invoke(&env, "echo", args).expect("invoke succeeds");
        assert_eq!(rendered, "hi");
    }
}
