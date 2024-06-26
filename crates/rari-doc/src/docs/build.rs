use std::borrow::Cow;

use rari_types::fm_types::PageType;
use rari_types::globals::{base_url, content_branch, git_history, popularities};
use rari_types::locale::Locale;
use scraper::Html;

use super::blog::BlogPost;
use super::curriculum::{
    build_landing_modules, build_overview_modules, build_sidebar, curriculum_group,
    prev_next_modules, prev_next_overview, CurriculumPage, Template,
};
use super::doc::{render_md_to_html, Doc};
use super::dummy::Dummy;
use super::json::{
    BuiltDocy, Compat, JsonBlogPost, JsonBlogPostDoc, JsonCurriculum, JsonDoADoc, JsonDoc, Prose,
    Section, Source, SpecificationSection, TocEntry,
};
use super::page::PageLike;
use super::parents::parents;
use super::sections::{split_sections, BuildSection, BuildSectionType};
use super::title::{page_title, transform_title};
use crate::baseline::get_baseline;
use crate::error::DocError;
use crate::html::rewriter::post_process_html;
use crate::html::sidebar::render_sidebar;
use crate::specs::extract_specifications;
use crate::templ::render::render;

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
}

pub fn make_toc(sections: &[BuildSection], with_h3: bool) -> Vec<TocEntry> {
    sections
        .iter()
        .filter_map(|section| section.make_toc_entry(with_h3))
        .collect()
}

pub fn build_content<T: PageLike>(doc: &T) -> Result<PageContent, DocError> {
    let ks_rendered_doc = if let Some(rari_env) = &doc.rari_env() {
        Cow::Owned(render(rari_env, doc.content())?)
    } else {
        Cow::Borrowed(doc.content())
    };
    let html = render_md_to_html(&ks_rendered_doc, doc.locale())?;
    let post_processed_html = post_process_html(&html, doc, false)?;
    let fragment = Html::parse_fragment(&post_processed_html);
    let (sections, summary) = split_sections(&fragment).expect("DOOM");
    let toc = make_toc(&sections, matches!(doc.page_type(), PageType::Curriculum));
    let body = sections.into_iter().map(Into::into).collect();
    Ok(PageContent { body, toc, summary })
}

pub fn build_doc(doc: &Doc) -> Result<BuiltDocy, DocError> {
    let PageContent { body, toc, summary } = build_content(doc)?;
    let sidebar_html = render_sidebar(doc)?;
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

    Ok(BuiltDocy::Doc(Box::new(JsonDoADoc {
        doc: JsonDoc {
            title: doc.title().to_string(),
            is_markdown: true,
            locale: doc.locale(),
            native: doc.locale().into(),
            mdn_url: doc.meta.url.clone(),
            is_translated: doc.meta.locale != Locale::default(),
            short_title,
            is_active: true,
            parents: parents(doc),
            page_title: page_title(doc, true)?,
            body,
            sidebar_html,
            toc,
            baseline,
            modified,
            summary,
            popularity,
            sidebar_macro: doc.meta.sidebar.first().cloned(),
            source: Source {
                folder,
                filename,
                github_url,
                last_commit_url,
            },
            browser_compat: doc.meta.browser_compat.clone(),
            ..Default::default()
        },
        url: doc.meta.url.clone(),
        ..Default::default()
    })))
}

pub fn build_blog_post(post: &BlogPost) -> Result<BuiltDocy, DocError> {
    let PageContent { body, toc, .. } = build_content(post)?;
    Ok(BuiltDocy::BlogPost(Box::new(JsonBlogPost {
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

pub fn build_dummy(dummy: &Dummy) -> Result<BuiltDocy, DocError> {
    dummy.as_built_doc()
}

pub fn build_curriculum(curriculum: &CurriculumPage) -> Result<BuiltDocy, DocError> {
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
    Ok(BuiltDocy::Curriculum(Box::new(JsonCurriculum {
        doc: super::json::JsonCurriculumDoc {
            title: curriculum.title().to_string(),
            locale: curriculum.locale(),
            native: curriculum.locale().into(),
            mdn_url: curriculum.meta.url.clone(),
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
