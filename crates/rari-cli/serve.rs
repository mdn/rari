use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, AtomicU64};

use axum::body::Body;
use axum::extract::{Path, Query, Request};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, put};
use axum::{Json, Router};
use rari_doc::cached_readers::wiki_histories;
use rari_doc::contributors::contributors_txt;
use rari_doc::error::{DocError, UrlError};
use rari_doc::issues::{IN_MEMORY, ISSUE_COUNTER_F, to_display_issues};
use rari_doc::pages::json::BuiltPage;
use rari_doc::pages::page::{Page, PageBuilder, PageCategory, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_doc::resolve::{UrlMeta, url_meta_from};
use rari_tools::error::ToolError;
use rari_tools::fix::issues::fix_page;
use rari_types::Popularities;
use rari_types::globals::{self, blog_root, content_root, content_translated_root};
use rari_types::locale::Locale;
use rari_utils::io::read_to_string;
use serde::Serialize;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use tracing::{Level, error, info, span};

static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);

static ASSET_EXTENSION: &[&str] = &[
    "gif", "jpeg", "jpg", "mp3", "mp4", "ogg", "png", "svg", "webm", "webp", "woff2",
];

tokio::task_local! {
    static SERVER_ISSUE_COUNTER: AtomicI64;
}

pub(crate) fn get_issue_counter_f() -> i64 {
    SERVER_ISSUE_COUNTER
        .try_with(|sic| sic.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        .unwrap_or(-1)
}

#[derive(Debug, Serialize)]
struct SearchItem {
    title: String,
    url: String,
}

async fn handler(req: Request) -> Response<Body> {
    let path = req.uri().path();
    if path.ends_with("/contributors.txt") {
        get_contributors_handler(req).await.into_response()
    } else if ASSET_EXTENSION.contains(
        &path
            .rsplit_once('.')
            .map(|(_, ext)| ext)
            .unwrap_or_default(),
    ) {
        get_file_handler(req).await.into_response()
    } else {
        get_json_handler(req).await.into_response()
    }
}

async fn wrapped_handler(req: Request) -> Response<Body> {
    SERVER_ISSUE_COUNTER
        .scope(AtomicI64::new(0), async move { handler(req).await })
        .await
}

fn fix_issues(params: HashMap<String, String>) -> Result<impl IntoResponse, AppError> {
    if let Some(url) = params.get("url") {
        tracing::info!("🔧 fixing {url}");
        let page = Page::from_url_with_fallback(url)?;
        fix_page(&page)?;
        Ok(Json("ok").into_response())
    } else {
        Ok((StatusCode::BAD_REQUEST).into_response())
    }
}

async fn wrapped_fix_issues(
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    SERVER_ISSUE_COUNTER
        .scope(AtomicI64::new(0), async move { fix_issues(params) })
        .await
}

async fn get_file_handler(req: Request) -> Result<Response, AppError> {
    let url = req.uri().path();
    tracing::info!("(asset) {}", url);
    let UrlMeta {
        page_category,
        slug,
        ..
    } = url_meta_from(url)?;

    // Blog author avatars are special.
    if matches!(page_category, PageCategory::BlogPost) && slug.starts_with("author/") {
        if let Some(blog_root_parent) = blog_root() {
            let path = blog_root_parent
                .join("authors")
                .join(slug.strip_prefix("author/").unwrap());
            return Ok(ServeFile::new(path).oneshot(req).await.into_response());
        }
    }

    if let Some(last_slash) = url.rfind('/') {
        let doc_url = &url[..(if matches!(
            page_category,
            PageCategory::BlogPost | PageCategory::Curriculum
        ) {
            // Add trailing slash for paths that require it.
            last_slash + 1
        } else {
            last_slash
        })];

        let file_name = &url[last_slash + 1..];
        let page = Page::from_url_with_fallback(doc_url)?;
        let path = page.full_path().with_file_name(file_name);

        return Ok(ServeFile::new(path).oneshot(req).await.into_response());
    }

    Ok((StatusCode::BAD_REQUEST).into_response())
}

async fn get_json_handler(req: Request) -> Result<Response, AppError> {
    let url = req.uri().path();
    let req_id = REQ_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let span = span!(Level::WARN, "serve", req = req_id);
    let _enter0 = span.enter();
    let span = span!(Level::ERROR, "url", "{}", url);
    let _enter1 = span.enter();
    let url = url.strip_suffix("/index.json").unwrap_or(url);
    match Page::from_url(url) {
        Ok(page) => {
            let file = page.full_path().to_string_lossy();
            let span = span!(
                Level::ERROR,
                "page",
                locale = page.locale().as_url_str(),
                slug = page.slug(),
                file = file.as_ref()
            );
            let _enter2 = span.enter();
            let mut json = page.build()?;
            tracing::info!("{url}");
            if let BuiltPage::Doc(json_doc) = &mut json {
                let m = IN_MEMORY.get_events();
                let (_, req_issues) = m
                    .remove(page.full_path().to_string_lossy().as_ref())
                    .unwrap_or_default();
                json_doc.doc.flaws = Some(to_display_issues(req_issues, &page));
            }
            Ok(Json(json).into_response())
        }
        Err(e @ (DocError::DocNotFound(..) | DocError::PageNotFound(..))) => {
            if let Some((doc_url, file_name)) = url.rsplit_once('/') {
                let page = Page::from_url_with_fallback(doc_url)?;
                let path = page.full_path().with_file_name(file_name);
                return Ok(ServeFile::new(path).oneshot(req).await.into_response());
            }
            Err(e.into())
        }
        Err(e) => {
            tracing::warn!("{e:?}");
            Err(e.into())
        }
    }
}

async fn get_contributors_handler(req: Request) -> impl IntoResponse {
    let url = req.uri().path();
    match get_contributors(url.strip_suffix("/contributors.txt").unwrap_or(url)) {
        Ok(contributors_txt_str) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain")],
            contributors_txt_str,
        )
            .into_response(),
        Err(e) => {
            tracing::error!("error generating contributors.txt for {url}: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

fn get_contributors(url: &str) -> Result<String, AppError> {
    let page = Page::from_url_with_fallback(url)?;
    let json = page.build()?;
    let github_file_url = if let BuiltPage::Doc(ref doc) = json {
        &doc.doc.source.github_url
    } else {
        ""
    };
    let wiki_histories = wiki_histories();
    let wiki_history = wiki_histories
        .get(&page.locale())
        .and_then(|wh| wh.get(page.slug()));
    let contributors_txt_str = contributors_txt(wiki_history, github_file_url);
    Ok(contributors_txt_str)
}

async fn get_search_index_handler(
    Path(locale): Path<String>,
) -> Result<Json<Vec<SearchItem>>, AppError> {
    tracing::info!("search index for: {locale}");
    let locale = Locale::from_str(&locale)?;
    Ok(Json(get_search_index(locale)?))
}

fn get_search_index(locale: Locale) -> Result<Vec<SearchItem>, DocError> {
    let in_file = globals::data_dir()
        .join("popularities")
        .join("popularities.json");
    let json_str = read_to_string(in_file)?;
    let popularities: Popularities = serde_json::from_str(&json_str)?;
    let docs = read_docs_parallel::<Page, Doc>(
        &[&if locale == Locale::EnUs {
            content_root()
        } else {
            content_translated_root().expect("no TRANSLATED_CONTENT_ROOT set")
        }
        .join(locale.as_folder_str())],
        None,
    )?;

    let mut index = docs
        .iter()
        .map(|doc| {
            (
                doc,
                popularities
                    .popularities
                    .get(doc.url())
                    .cloned()
                    .unwrap_or_default(),
            )
        })
        .collect::<Vec<(&Page, f64)>>();
    index.sort_by(|(da, a), (db, b)| match b.partial_cmp(a) {
        None | Some(Ordering::Equal) => da.title().cmp(db.title()),
        Some(ord) => ord,
    });
    let out = index
        .into_iter()
        .map(|(doc, _)| SearchItem {
            title: doc.title().to_string(),
            url: doc.url().to_string(),
        })
        .collect::<Vec<_>>();

    Ok(out)
}

#[derive(Debug)]
struct AppError(ToolError);

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        match self.0 {
            ToolError::DocError(
                DocError::RariIoError(_)
                | DocError::IOError(_)
                | DocError::PageNotFound(..)
                | DocError::UrlError(UrlError::InvalidUrl),
            ) => (StatusCode::NOT_FOUND, "").into_response(),

            _ => (StatusCode::INTERNAL_SERVER_ERROR, error!("🤷: {}", self.0)).into_response(),
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<ToolError>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn serve() -> Result<(), anyhow::Error> {
    ISSUE_COUNTER_F.get_or_init(|| get_issue_counter_f)();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let app = Router::new()
                .route("/_document/fixfixableflaws", put(wrapped_fix_issues))
                .route("/{locale}/search-index.json", get(get_search_index_handler))
                .fallback(wrapped_handler);

            const PORT: u16 = 8083;
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}"))
                .await
                .map_err(|e| {
                    error!("Failed to bind to port {PORT}: {}", e);
                    e
                })
                .unwrap();

            info!("Rari server started on http://0.0.0.0:{PORT}");
            axum::serve(listener, app).await.unwrap();
        });
    Ok(())
}
