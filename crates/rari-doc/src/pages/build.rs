use std::borrow::Cow;
use std::fs;
use std::path::Path;

use rari_md::m2h;
use rari_types::fm_types::PageType;
use rari_types::globals::{base_url, content_branch, git_history, popularities};
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use scraper::Html;

use super::json::{
    BuiltPage, Compat, ContributorSpotlightHyData, JsonBlogPostDoc, JsonBlogPostPage,
    JsonCurriculumPage, JsonDoc, JsonDocPage, JsonGenericHyData, JsonGenericPage, Prose, Section,
    Source, SpecificationSection, TocEntry, Translation,
};
use super::page::{Page, PageBuilder, PageLike};
use super::types::contributors::ContributorSpotlight;
use super::types::generic::GenericPage;
use crate::baseline::get_baseline;
use crate::error::DocError;
use crate::helpers::parents::parents;
use crate::helpers::title::{page_title, transform_title};
use crate::html::bubble_up::bubble_up_curriculum_page;
use crate::html::modifier::{add_missing_ids, remove_empty_p};
use crate::html::rewriter::{post_process_html, post_process_inline_sidebar};
use crate::html::sections::{split_sections, BuildSection, BuildSectionType, Splitted};
use crate::html::sidebar::{
    build_sidebars, expand_details_and_mark_current_for_inline_sidebar, postprocess_sidebar,
};
use crate::pages::json::JsonContributorSpotlightPage;
use crate::pages::types::blog::BlogPost;
use crate::pages::types::curriculum::{
    build_landing_modules, build_overview_modules, build_sidebar, curriculum_group,
    prev_next_modules, prev_next_overview, CurriculumPage, Template,
};
use crate::pages::types::doc::Doc;
use crate::pages::types::spa::SPA;
use crate::specs::extract_specifications;
use crate::templ::render::{decode_ref, render, Rendered};
use crate::translations::get_translations_for;

impl<'a> From<BuildSection<'a>> for Section {
    fn from(value: BuildSection) -> Self {
        match value.typ {
            BuildSectionType::Prose | BuildSectionType::Unknown => Self::Prose(Prose {
                title: value.heading.map(|h| h.inner_html()),
                content: value.body.join("\n"),
                is_h3: value.is_h3,
                id: value.id,
            }),
            BuildSectionType::Specification => {
                let title = value
                    .heading
                    .map(|h| h.inner_html())
                    .or_else(|| value.query.clone());
                let id = value.id.or_else(|| value.query.clone());
                let specifications = extract_specifications(
                    &value
                        .query
                        .as_ref()
                        .map(|q| {
                            q.split(',')
                                .map(String::from)
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                    &value
                        .spec_urls
                        .map(|q| {
                            q.split(',')
                                .map(String::from)
                                .filter(|s| !s.is_empty())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                );
                let query = value.query.unwrap_or_default();
                let query = if query.is_empty() {
                    "undefined".to_string()
                } else {
                    query
                };
                Self::Specifications(SpecificationSection {
                    id,
                    title,
                    is_h3: value.is_h3,
                    specifications,
                    query,
                    content: if value.body.is_empty() {
                        None
                    } else {
                        Some(value.body.join("\n"))
                    },
                })
            }
            BuildSectionType::Compat => {
                let title = value
                    .heading
                    .map(|h| h.inner_html())
                    .or_else(|| value.query.clone());
                let id = value.id.or_else(|| value.query.clone());
                Self::BrowserCompatibility(Compat {
                    title,
                    is_h3: value.is_h3,
                    id,
                    query: value.query.unwrap_or_default(),
                    content: if value.body.is_empty() {
                        None
                    } else {
                        Some(value.body.join("\n"))
                    },
                })
            }
        }
    }
}

impl<'a> BuildSection<'a> {
    pub fn make_toc_entry(&self, with_h3: bool) -> Option<TocEntry> {
        let id = self.id.clone();
        let text = self.heading.map(|h| h.inner_html());
        if let (Some(id), Some(text)) = (id, text) {
            if !self.is_h3 || with_h3 {
                return Some(TocEntry { id, text });
            }
        }
        None
    }
}

pub struct PageContent {
    body: Vec<Section>,
    toc: Vec<TocEntry>,
    summary: Option<String>,
    sidebar: Option<String>,
}

pub fn make_toc(sections: &[BuildSection], with_h3: bool) -> Vec<TocEntry> {
    sections
        .iter()
        .filter_map(|section| section.make_toc_entry(with_h3))
        .collect()
}

fn build_content<T: PageLike>(page: &T) -> Result<PageContent, DocError> {
    let (ks_rendered_doc, templs, sidebars) = if let Some(rari_env) = &page.rari_env() {
        let Rendered {
            content,
            templs,
            sidebars,
        } = render(rari_env, page.content(), page.fm_offset())?;
        (Cow::Owned(content), templs, sidebars)
    } else {
        (Cow::Borrowed(page.content()), vec![], vec![])
    };
    let encoded_html = m2h(&ks_rendered_doc, page.locale())?;
    let html = decode_ref(&encoded_html, &templs)?;
    let post_processed_html = post_process_html(&html, page, false)?;

    let mut fragment = Html::parse_fragment(&post_processed_html);
    if page.page_type() == PageType::Curriculum {
        bubble_up_curriculum_page(&mut fragment)?;
    }
    remove_empty_p(&mut fragment)?;
    add_missing_ids(&mut fragment)?;
    expand_details_and_mark_current_for_inline_sidebar(&mut fragment, page.url())?;
    let Splitted {
        sections,
        summary,
        sidebar,
    } = split_sections(&fragment).expect("DOOM");

    // TODO cleanup
    let mut sidebars = sidebars
        .iter()
        .map(|s| postprocess_sidebar(s, page))
        .collect::<Vec<_>>();
    if let Some(sidebar) = &sidebar {
        sidebars.push(post_process_inline_sidebar(sidebar));
    }

    let sidebar = if sidebars.is_empty() {
        None
    } else {
        Some(sidebars.into_iter().collect::<Result<String, _>>()?)
    };
    let toc = make_toc(&sections, matches!(page.page_type(), PageType::Curriculum));
    let body = sections.into_iter().map(Into::into).collect();
    Ok(PageContent {
        body,
        toc,
        summary,
        sidebar,
    })
}

fn build_doc(doc: &Doc) -> Result<BuiltPage, DocError> {
    let PageContent {
        body,
        toc,
        summary,
        sidebar,
    } = build_content(doc)?;
    let sidebar_html = if sidebar.is_some() {
        sidebar
    } else {
        build_sidebars(doc)?
    };
    let baseline = get_baseline(&doc.meta.browser_compat);
    let folder = doc
        .meta
        .path
        .parent()
        .unwrap_or(&doc.meta.path)
        .to_path_buf();
    let filename = doc
        .meta
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let history = git_history().get(&doc.meta.path);
    let modified = history.map(|entry| entry.modified).unwrap_or_default();
    let short_title = doc
        .short_title()
        .map(String::from)
        .unwrap_or(transform_title(doc.title()).to_string());

    let github_url = format!(
        "https://github.com/mdn/{}/blob/{}/files/{}",
        if doc.locale() == Locale::default() {
            "content"
        } else {
            "translated-content"
        },
        content_branch(),
        doc.meta.path.display()
    );

    let last_commit_url = format!(
        "https://github.com/mdn/{}/commit/{}",
        if doc.locale() == Locale::default() {
            "content"
        } else {
            "translated-content"
        },
        history.map(|entry| entry.hash.as_str()).unwrap_or_default()
    );
    let popularity = popularities().popularities.get(doc.url()).cloned();
    let other_translations = get_translations_for(doc.slug(), doc.locale())
        .into_iter()
        .map(|(locale, title)| Translation {
            native: locale.into(),
            locale,
            title,
        })
        .collect();

    let no_indexing =
        doc.meta.slug == "MDN/Kitchensink" || doc.is_orphaned() || doc.is_conflicting();
    let parents = if !doc.is_conflicting() && !doc.is_orphaned() {
        parents(doc)
    } else {
        Default::default()
    };

    Ok(BuiltPage::Doc(Box::new(JsonDocPage {
        doc: JsonDoc {
            title: doc.title().to_string(),
            is_markdown: true,
            locale: doc.locale(),
            native: doc.locale().into(),
            mdn_url: doc.meta.url.clone(),
            is_translated: doc.meta.locale != Locale::default(),
            short_title,
            is_active: true,
            parents,
            page_title: page_title(doc, true)?,
            body,
            sidebar_html,
            toc,
            baseline,
            modified,
            summary,
            popularity,
            no_indexing,
            sidebar_macro: doc.meta.sidebar.first().cloned(),
            source: Source {
                folder,
                filename,
                github_url,
                last_commit_url,
            },
            browser_compat: doc.meta.browser_compat.clone(),
            other_translations,
            page_type: doc.meta.page_type,
        },
        url: doc.meta.url.clone(),
    })))
}

fn build_blog_post(post: &BlogPost) -> Result<BuiltPage, DocError> {
    let PageContent { body, toc, .. } = build_content(post)?;
    Ok(BuiltPage::BlogPost(Box::new(JsonBlogPostPage {
        doc: JsonBlogPostDoc {
            title: post.title().to_string(),
            mdn_url: post.url().to_owned(),
            native: post.locale().into(),
            page_title: page_title(post, true)?,
            locale: post.locale(),
            body,
            toc,
            summary: Some(post.meta.description.clone()),
            ..Default::default()
        },
        url: post.url().to_owned(),
        locale: post.locale(),
        blog_meta: Some((&post.meta).into()),
        page_title: page_title(post, false)?,
        image: Some(format!(
            "{}{}{}",
            base_url(),
            post.url(),
            post.meta.image.file
        )),
        ..Default::default()
    })))
}

fn build_generic_page(page: &GenericPage) -> Result<BuiltPage, DocError> {
    let built = build_content(page);
    let PageContent { body, toc, .. } = built?;
    Ok(BuiltPage::GenericPage(Box::new(JsonGenericPage {
        hy_data: JsonGenericHyData {
            sections: body,
            title: page.meta.title.clone(),
            toc,
        },
        page_title: concat_strs!(
            page.meta.title.as_str(),
            " | ",
            page.meta.title_suffix.as_str()
        ),
        url: page.meta.url.clone(),
        id: page.meta.page.clone(),
    })))
}

fn build_spa(spa: &SPA) -> Result<BuiltPage, DocError> {
    spa.as_built_doc()
}

fn build_curriculum(curriculum: &CurriculumPage) -> Result<BuiltPage, DocError> {
    let PageContent { body, toc, .. } = build_content(curriculum)?;
    let sidebar = build_sidebar().ok();
    let parents = parents(curriculum);
    let group = curriculum_group(&parents);
    let modules = match curriculum.meta.template {
        Template::Overview => build_overview_modules(curriculum.slug())?,
        Template::Landing => build_landing_modules()?,
        _ => Default::default(),
    };
    let prev_next = match curriculum.meta.template {
        Template::Module => prev_next_modules(curriculum.slug())?,
        Template::Overview => prev_next_overview(curriculum.slug())?,
        _ => None,
    };
    Ok(BuiltPage::Curriculum(Box::new(JsonCurriculumPage {
        doc: super::json::JsonCurriculumDoc {
            title: curriculum.title().to_string(),
            locale: curriculum.locale(),
            native: curriculum.locale().into(),
            mdn_url: curriculum.meta.url.clone(),
            template: curriculum.meta.template,
            parents,
            page_title: page_title(curriculum, true)?,
            summary: curriculum.meta.summary.clone(),
            body,
            sidebar,
            toc,
            group,
            modules,
            prev_next,
            topic: Some(curriculum.meta.topic),
            ..Default::default()
        },
        url: curriculum.url().to_owned(),
        page_title: page_title(curriculum, false)?,
        locale: curriculum.locale(),
    })))
}

fn build_contributor_spotlight(cs: &ContributorSpotlight) -> Result<BuiltPage, DocError> {
    let PageContent { body, .. } = build_content(cs)?;
    let contributor_spotlight_data = ContributorSpotlightHyData {
        sections: body,
        contributor_name: cs.meta.contributor_name.clone(),
        folder_name: cs.meta.folder_name.clone(),
        is_featured: cs.meta.is_featured,
        profile_img: cs.meta.img.clone(),
        profile_img_alt: cs.meta.img_alt.clone(),
        usernames: cs.meta.usernames.clone(),
        quote: cs.meta.quote.clone(),
    };
    Ok(BuiltPage::ContributorSpotlight(Box::new(
        JsonContributorSpotlightPage {
            url: cs.meta.url.clone(),
            page_title: cs.meta.title.clone(),
            hy_data: contributor_spotlight_data,
        },
    )))
}

/// Copies additional files from one directory to another, excluding a specified file.
///
/// This function reads all files from the source directory, filters out the specified file to ignore,
/// and copies the remaining files to the destination directory. This is useful for copying additional
/// assets from a source directory to a destination directory, ususally excluding the original `index.md`
/// file.
///
/// # Arguments
///
/// * `from` - A reference to a `Path` that specifies the source directory.
/// * `to` - A reference to a `Path` that specifies the destination directory.
/// * `ignore` - A reference to a `Path` that specifies the file to be ignored during the copy process.
///
/// # Returns
///
/// * `Result<(), DocError>` - Returns `Ok(())` if all files are copied successfully, or a `DocError` if an error occurs.
pub(crate) fn copy_additional_files(from: &Path, to: &Path, ignore: &Path) -> Result<(), DocError> {
    for from in fs::read_dir(from)?
        .filter_map(Result::ok)
        .map(|f| f.path())
        .filter(|p| p.is_file() && p != ignore)
    {
        if let Some(filename) = from.file_name() {
            let to = to.to_path_buf().join(filename);
            fs::copy(&from, to)?;
        }
    }
    Ok(())
}

impl PageBuilder for Page {
    fn build(&self) -> Result<BuiltPage, DocError> {
        match self {
            Self::Doc(doc) => build_doc(doc),
            Self::BlogPost(post) => build_blog_post(post),
            Self::SPA(spa) => build_spa(spa),
            Self::Curriculum(curriculum) => build_curriculum(curriculum),
            Self::ContributorSpotlight(cs) => build_contributor_spotlight(cs),
            Self::GenericPage(generic) => build_generic_page(generic),
        }
    }
}
