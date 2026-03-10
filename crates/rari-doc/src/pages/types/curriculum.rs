use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use constcat::concat;
use rari_types::RariEnv;
use rari_types::fm_types::{FeatureStatus, PageType};
use rari_types::globals::curriculum_root;
use rari_types::locale::Locale;
use rari_utils::io::read_to_string;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::cached_readers::curriculum_files;
use crate::error::DocError;
use crate::pages::json::{Parent, PrevNextBySlug, PrevNextByUrl, UrlNTitle};
use crate::pages::page::{Page, PageCategory, PageLike, PageReader};
use crate::pages::types::utils::FmTempl;
use crate::utils::{as_null, split_fm};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Template {
    Module,
    Overview,
    Landing,
    About,
    #[default]
    #[serde(other)]
    Default,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default, JsonSchema)]
pub enum Topic {
    #[serde(rename = "Web Standards & Semantics")]
    WebStandards,
    Styling,
    Scripting,
    #[serde(rename = "Best Practices")]
    BestPractices,
    Tooling,
    #[default]
    #[serde(other, serialize_with = "as_null", untagged)]
    None,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumSidebarEntry {
    pub url: String,
    pub title: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CurriculumSidebarEntry>,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumIndexEntry {
    pub url: String,
    pub title: String,
    pub slug: Option<String>,
    pub summary: Option<String>,
    pub topic: Topic,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<CurriculumIndexEntry>,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)]
pub struct CurriculumFrontmatter {
    pub summary: Option<String>,
    pub template: Template,
    pub topic: Topic,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumBuildMeta {
    pub url: String,
    pub title: String,
    pub slug: String,
    pub summary: Option<String>,
    pub template: Template,
    pub topic: Topic,
    pub filename: PathBuf,
    pub full_path: PathBuf,
    pub path: PathBuf,
    pub group: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurriculumMeta {
    pub url: String,
    pub title: String,
    pub slug: Option<String>,
    pub summary: Option<String>,
    pub template: Template,
    pub topic: Topic,
    pub sidebar: Vec<CurriculumIndexEntry>,
    pub modules: Vec<CurriculumIndexEntry>,
    pub parents: Vec<Parent>,
    pub prev_next: PrevNextBySlug,
    pub group: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Curriculum {
    pub meta: CurriculumBuildMeta,
    raw_content: String,
}

impl Curriculum {
    pub fn page_from_url(url: &str) -> Option<Page> {
        let _ = curriculum_root()?;
        curriculum_files()
            .by_url
            .get(&url.to_ascii_lowercase())
            .cloned()
    }

    pub fn page_from_file_path(path: &Path) -> Option<Page> {
        let _ = curriculum_root()?;
        curriculum_files().by_path.get(path).cloned()
    }

    pub fn page_from_relative_file(
        base_file: &Path,
        relative_file: &str,
    ) -> Result<Page, DocError> {
        let mut path = base_file
            .parent()
            .ok_or(DocError::NoParent(base_file.to_path_buf()))?
            .to_path_buf()
            .join(relative_file);
        if path.is_dir() {
            path = path.join("0-README.md");
        }
        let path = fs::canonicalize(path)?;
        Curriculum::page_from_file_path(&path).ok_or(DocError::PageNotFound(
            path.to_string_lossy().to_string(),
            PageCategory::Curriculum,
        ))
    }
}

impl PageReader<Page> for Curriculum {
    fn read(path: impl Into<PathBuf>, _: Option<Locale>) -> Result<Page, DocError> {
        let full_path = path.into();
        let raw = read_to_string(&full_path)?;
        let (fm, content_start) = split_fm(&raw);
        let fm = fm.ok_or(DocError::NoFrontmatter)?;

        let raw_content = &raw[content_start..];
        let curriculum_dir = curriculum_root()
            .ok_or(DocError::NoCurriculumRoot)?
            .join("curriculum");
        let filename = full_path.strip_prefix(&curriculum_dir)?.to_owned();
        let slug = curriculum_file_to_slug(&filename);
        let url = if slug.is_empty() {
            format!("/{}/curriculum/", Locale::default().as_url_str())
        } else {
            format!("/{}/curriculum/{slug}/", Locale::default().as_url_str())
        };
        let (title, line) = TITLE_RE
            .captures(raw_content)
            .map(|cap| (cap[1].to_owned(), cap[0].to_owned()))
            .ok_or(DocError::NoH1)?;
        let raw_content = raw_content.replacen(&line, "", 1);
        let CurriculumFrontmatter {
            summary,
            template,
            topic,
        } = serde_yaml_ng::from_str(fm)?;
        let path = full_path.strip_prefix(&curriculum_dir)?.to_path_buf();
        let meta = CurriculumBuildMeta {
            url,
            title,
            slug,
            summary,
            template,
            topic,
            filename,
            full_path,
            path,
            group: None,
        };
        let page = Page::Curriculum(Arc::new(Curriculum { meta, raw_content }));
        Ok(page)
    }
}

static TITLE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^[\w\n]*#\s+(.*)\n"#).unwrap());
static SLUG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(\d+-|\.md$|\/?0?-?README)"#).unwrap());

fn curriculum_file_to_slug(file: &Path) -> String {
    SLUG_RE.replace_all(&file.to_string_lossy(), "").to_string()
}

impl PageLike for Curriculum {
    fn url(&self) -> &str {
        &self.meta.url
    }

    fn slug(&self) -> &str {
        &self.meta.slug
    }

    fn title(&self) -> &str {
        &self.meta.title
    }

    fn short_title(&self) -> Option<&str> {
        None
    }

    fn locale(&self) -> Locale {
        Default::default()
    }

    fn content(&self) -> &str {
        &self.raw_content
    }

    fn rari_env(&self) -> Option<RariEnv<'_>> {
        None
    }

    fn render(&self) -> Result<String, DocError> {
        todo!()
    }

    fn title_suffix(&self) -> Option<&str> {
        Some("MDN Curriculum")
    }

    fn page_type(&self) -> PageType {
        PageType::Curriculum
    }

    fn status(&self) -> &[FeatureStatus] {
        &[]
    }

    fn full_path(&self) -> &Path {
        &self.meta.full_path
    }

    fn path(&self) -> &Path {
        &self.meta.path
    }

    fn base_slug(&self) -> &str {
        concat!("/", Locale::EnUs.as_url_str(), "/")
    }

    fn trailing_slash(&self) -> bool {
        true
    }

    fn fm_offset(&self) -> usize {
        0
    }

    fn raw_content(&self) -> &str {
        &self.raw_content
    }

    fn banners(&self) -> Option<&[FmTempl]> {
        None
    }
}

pub fn curriculum_group(parents: &[Parent]) -> Option<String> {
    if parents.len() > 1
        && let Some(group) = parents.get(parents.len() - 2)
        && group.title.ends_with("modules")
    {
        return Some(group.title.to_string());
    };
    None
}

pub fn build_sidebar() -> Result<Vec<CurriculumSidebarEntry>, DocError> {
    let mut sidebar: Vec<(PathBuf, CurriculumSidebarEntry)> = curriculum_files()
        .by_path
        .values()
        .map(|c| {
            (
                c.full_path().to_path_buf(),
                CurriculumSidebarEntry {
                    url: c.url().to_string(),
                    title: c.title().to_string(),
                    slug: c.slug().to_string(),
                    children: Vec::new(),
                },
            )
        })
        .collect();
    sidebar.sort_by(|a, b| a.0.cmp(&b.0));
    let sidebar = sidebar.into_iter().fold(
        Vec::new(),
        |mut acc: Vec<CurriculumSidebarEntry>, (_, entry)| {
            let lvl = entry.slug.split('/').count();
            if lvl > 2
                && let Some(last) = acc.last_mut()
            {
                last.children.push(entry);
                return acc;
            }

            acc.push(entry);
            acc
        },
    );

    Ok(sidebar)
}

pub fn build_landing_modules() -> Result<Vec<CurriculumIndexEntry>, DocError> {
    Ok(grouped_index()?
        .iter()
        .filter(|m| !m.children.is_empty())
        .cloned()
        .collect())
}

pub fn build_overview_modules(slug: &str) -> Result<Vec<CurriculumIndexEntry>, DocError> {
    Ok(grouped_index()?
        .iter()
        .filter_map(|m| {
            if m.slug.as_deref() == Some(slug) {
                Some(m.children.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect())
}

pub fn prev_next_modules(slug: &str) -> Result<Option<PrevNextByUrl>, DocError> {
    let index = &curriculum_files().index;
    let i = index
        .iter()
        .position(|entry| entry.slug.as_deref() == Some(slug));
    prev_next(index, i)
}

pub fn prev_next_overview(slug: &str) -> Result<Option<PrevNextByUrl>, DocError> {
    let index: Vec<_> = grouped_index()?
        .into_iter()
        .filter_map(|entry| {
            if entry.children.is_empty() {
                None
            } else {
                Some(entry)
            }
        })
        .collect();
    let i = index
        .iter()
        .position(|entry| entry.slug.as_deref() == Some(slug));
    prev_next(&index, i)
}

pub fn prev_next(
    index: &[CurriculumIndexEntry],
    i: Option<usize>,
) -> Result<Option<PrevNextByUrl>, DocError> {
    Ok(i.map(|i| match i {
        0 => PrevNextByUrl {
            prev: None,
            next: index.get(1).map(|entry| UrlNTitle {
                title: entry.title.clone(),
                url: entry.url.clone(),
            }),
        },
        i if i == index.len() => PrevNextByUrl {
            prev: index.get(i - 1).map(|entry| UrlNTitle {
                title: entry.title.clone(),
                url: entry.url.clone(),
            }),
            next: None,
        },

        i => PrevNextByUrl {
            prev: index.get(i - 1).map(|entry| UrlNTitle {
                title: entry.title.clone(),
                url: entry.url.clone(),
            }),
            next: index.get(i + 1).map(|entry| UrlNTitle {
                title: entry.title.clone(),
                url: entry.url.clone(),
            }),
        },
    }))
}

fn grouped_index() -> Result<Vec<CurriculumIndexEntry>, DocError> {
    Ok(curriculum_files().index.iter().fold(
        Vec::new(),
        |mut acc: Vec<CurriculumIndexEntry>, entry| {
            let lvl = entry.slug.as_deref().unwrap_or_default().split('/').count();
            if lvl > 2
                && let Some(last) = acc.last_mut()
            {
                last.children.push(entry.clone());
                return acc;
            }

            acc.push(entry.clone());
            acc
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_file_to_slug_root_readme() {
        // Root README should produce empty slug
        assert_eq!(curriculum_file_to_slug(Path::new("0-README.md")), "");
    }

    #[test]
    fn test_curriculum_file_to_slug_simple_module() {
        // Simple module path
        assert_eq!(
            curriculum_file_to_slug(Path::new("1-intro/0-README.md")),
            "intro"
        );
    }

    #[test]
    fn test_curriculum_file_to_slug_nested_path() {
        // Nested path with multiple numeric prefixes
        assert_eq!(
            curriculum_file_to_slug(Path::new("2-core/3-modules/1-something.md")),
            "core/modules/something"
        );
    }

    #[test]
    fn test_curriculum_file_to_slug_deep_nested() {
        // Deep nested path
        assert_eq!(
            curriculum_file_to_slug(Path::new(
                "1-getting-started/2-environment-setup/0-README.md"
            )),
            "getting-started/environment-setup"
        );
    }

    #[test]
    fn test_curriculum_file_to_slug_two_digit_prefix() {
        // Two-digit numeric prefix
        assert_eq!(
            curriculum_file_to_slug(Path::new("10-advanced/05-topics.md")),
            "advanced/topics"
        );
    }

    #[test]
    fn test_curriculum_file_to_slug_readme_without_zero_prefix() {
        // README without zero prefix
        assert_eq!(
            curriculum_file_to_slug(Path::new("1-section/README.md")),
            "section"
        );
    }

    #[test]
    fn test_curriculum_file_to_slug_preserves_hyphens_in_names() {
        // Hyphens in names should be preserved
        assert_eq!(
            curriculum_file_to_slug(Path::new("1-web-standards/2-html-basics.md")),
            "web-standards/html-basics"
        );
    }
}
