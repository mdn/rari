use std::cmp::Ordering;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rari_doc::error::DocError;
use rari_doc::issues::{to_display_issues, InMemoryLayer};
use rari_doc::pages::json::BuiltPage;
use rari_doc::pages::page::{Page, PageBuilder, PageLike};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_types::globals::{content_root, content_translated_root};
use rari_types::locale::Locale;
use rari_types::Popularities;
use rari_utils::io::read_to_string;
use serde::Serialize;
use tracing::{error, span, Level};

static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Serialize)]
struct SearchItem {
    title: String,
    url: String,
}
async fn get_json_handler(
    State(memory_layer): State<Arc<InMemoryLayer>>,
    req: Request,
) -> Result<Json<BuiltPage>, AppError> {
    let req_id = REQ_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let span = span!(Level::WARN, "serve", req = req_id);
    let _enter1 = span.enter();
    let url = req.uri().path();
    let mut json = get_json(url)?;
    if let BuiltPage::Doc(json_doc) = &mut json {
        let m = memory_layer.get_events();
        let mut issues = m.lock().unwrap();
        let req_isses: Vec<_> = issues
            .iter()
            .filter(|issue| issue.req == req_id)
            .cloned()
            .collect();
        issues.retain_mut(|i| i.req != req_id);
        json_doc.doc.flaws = Some(to_display_issues(req_isses));
    }
    Ok(Json(json))
}

fn get_json(url: &str) -> Result<BuiltPage, DocError> {
    let span = span!(Level::ERROR, "url", "{}", url);
    let _enter1 = span.enter();
    let url = url.strip_suffix("/index.json").unwrap_or(url);
    let page = Page::from_url_with_fallback(url)?;

    let slug = &page.slug();
    let locale = page.locale();
    let span = span!(Level::ERROR, "page", "{}:{}", locale, slug);
    let _enter2 = span.enter();
    let json = page.build()?;
    tracing::info!("{url}");
    Ok(json)
}

async fn get_search_index_handler(
    Path(locale): Path<String>,
) -> Result<Json<Vec<SearchItem>>, AppError> {
    tracing::info!("search index for: {locale}");
    let locale = Locale::from_str(&locale)?;
    Ok(Json(get_search_index(locale)?))
}

fn get_search_index(locale: Locale) -> Result<Vec<SearchItem>, DocError> {
    let in_file = content_root()
        .join(Locale::EnUs.as_folder_str())
        .join("popularities.json");
    let json_str = read_to_string(in_file)?;
    let popularities: Popularities = serde_json::from_str(&json_str)?;
    let docs = read_docs_parallel::<Doc>(
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

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, error!("🤷: {}", self.0)).into_response()
    }
}
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn serve(memory_layer: InMemoryLayer) -> Result<(), anyhow::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let app = Router::new()
                .route("/:locale/search-index.json", get(get_search_index_handler))
                .fallback(get_json_handler)
                .with_state(Arc::new(memory_layer));

            let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    Ok(())
}
