use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use concat_in_place::strcat;
use rari_types::globals::{content_root, content_translated_root};
use rari_types::locale::Locale;
use regex::Regex;

use crate::cached_readers::contributor_spotlight_files;
use crate::error::DocError;
use crate::helpers::summary_hack::get_hacky_summary_md;
use crate::pages::json::{
    HomePageFeaturedArticle, HomePageFeaturedContributor, HomePageLatestNewsItem,
    HomePageRecentContribution, NameUrl, Parent,
};
use crate::pages::page::{url_path_to_page_with_other_locale_and_fallback, Page, PageLike};
use crate::pages::parents::parents;

pub fn lastet_news(urls: &[&str]) -> Result<Vec<HomePageLatestNewsItem>, DocError> {
    urls.iter()
        .filter_map(|url| match Page::page_from_url_path(url) {
            Ok(Page::BlogPost(post)) => Some(Ok(HomePageLatestNewsItem {
                url: post.url().to_string(),
                title: post.title().to_string(),
                author: Some(post.meta.author.clone()),
                source: NameUrl {
                    name: "developer.mozilla.org".to_string(),
                    url: strcat!("/" Locale::default().as_url_str() "/blog/"),
                },
                published_at: post.meta.date,
            })),
            Err(e) => Some(Err(e)),
            x => {
                tracing::debug!("{x:?}");
                None
            }
        })
        .collect()
}

pub fn featured_articles(
    urls: &[&str],
    locale: Locale,
) -> Result<Vec<HomePageFeaturedArticle>, DocError> {
    urls.iter()
        .filter_map(|url| {
            match url_path_to_page_with_other_locale_and_fallback(url, Some(locale)) {
                Ok(Page::BlogPost(post)) => Some(Ok(HomePageFeaturedArticle {
                    mdn_url: post.url().to_string(),
                    summay: post.meta.description.clone(),
                    title: post.title().to_string(),
                    tag: Some(Parent {
                        uri: strcat!("/" Locale::default().as_url_str() "/blog/"),
                        title: "Blog".to_string(),
                    }),
                })),
                Ok(ref page @ Page::Doc(ref doc)) => Some(Ok(HomePageFeaturedArticle {
                    mdn_url: doc.url().to_string(),
                    summay: get_hacky_summary_md(page).unwrap_or_default(),
                    title: doc.title().to_string(),
                    tag: parents(page).get(1).cloned(),
                })),
                Err(e) => Some(Err(e)),
                x => {
                    tracing::debug!("{x:?}");
                    None
                }
            }
        })
        .collect()
}

pub fn recent_contributions() -> Result<Vec<HomePageRecentContribution>, DocError> {
    let mut content = recent_contributions_from_git(content_root(), "mdn/content")?;
    if let Some(translated_root) = content_translated_root() {
        content.extend(recent_contributions_from_git(
            translated_root,
            "mdn/translated_content",
        )?);
    };
    content.sort_by(|a, b| a.updated_at.cmp(&b.updated_at));
    Ok(content)
}

static GIT_LOG_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^(?<date>[^ ]+) (?<msg>.*[^\)])( \(#(?<pr>\d+)\))?$"#).unwrap());

fn recent_contributions_from_git(
    path: &Path,
    repo: &str,
) -> Result<Vec<HomePageRecentContribution>, DocError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path)
        .output()
        .expect("failed to execute git rev-parse");

    let repo_root_raw = String::from_utf8_lossy(&output.stdout);
    let repo_root = repo_root_raw.trim();

    let output = Command::new("git")
        .args([
            "log",
            "--no-merges",
            "--pretty=format:%aI %s",
            "-n 10",
            "-z",
        ])
        .current_dir(repo_root)
        .output()
        .expect("failed to execute process");

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str
        .split(['\0'])
        .filter_map(|line| {
            GIT_LOG_LINE.captures(line.trim()).and_then(|cap| {
                match (cap.name("date"), cap.name("msg"), cap.name("pr")) {
                    (Some(date), Some(msg), Some(pr)) => Some(HomePageRecentContribution {
                        number: pr.as_str().parse::<i64>().unwrap_or_default(),
                        title: msg.as_str().to_string(),
                        updated_at: date.as_str().parse::<DateTime<Utc>>().unwrap_or_default(),
                        url: strcat!("https://github.com/" repo "/pull/" pr.as_str()),
                        repo: NameUrl {
                            name: repo.to_string(),
                            url: strcat!("https://github.com/" repo),
                        },
                    }),
                    _ => None,
                }
            })
        })
        .take(5)
        .collect())
}

pub fn featured_contributor(
    locale: Locale,
) -> Result<Option<HomePageFeaturedContributor>, DocError> {
    Ok(contributor_spotlight_files()
        .values()
        .find_map(|cs| {
            if let Page::ContributorSpotlight(cs) = cs {
                if cs.meta.is_featured && cs.locale() == locale {
                    Some(cs)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .map(|cs| HomePageFeaturedContributor {
            contributor_name: cs.meta.contributor_name.clone(),
            url: cs.url().to_string(),
            quote: cs.meta.quote.clone(),
        }))
}
