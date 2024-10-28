use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use rari_doc::error::DocError;
use rari_doc::issues::{to_display_issues, InMemoryLayer};
use rari_doc::pages::json::BuiltPage;
use rari_doc::pages::page::{Page, PageBuilder, PageLike};
use tracing::{error, span, Level};

static REQ_COUNTER: AtomicU64 = AtomicU64::new(1);

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

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, error!("🤷‍♂️: {}", self.0)).into_response()
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
                .fallback(get_json_handler)
                .with_state(Arc::new(memory_layer));

            let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    Ok(())
}
