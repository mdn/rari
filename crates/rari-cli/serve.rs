use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use rari_doc::pages::json::BuiltDocy;
use rari_doc::pages::page::{Page, PageBuilder, PageLike};
use tracing::{error, span, Level};

async fn get_json(req: Request) -> Result<Json<BuiltDocy>, AppError> {
    let url = req.uri().path();
    let span = span!(Level::ERROR, "url", "{}", url);
    let _enter1 = span.enter();
    let url = url.strip_suffix("index.json").unwrap_or(url);
    let page = Page::from_url(url)?;

    let slug = &page.slug();
    let locale = page.locale();
    let span = span!(Level::ERROR, "page", "{}:{}", locale, slug);
    let _enter2 = span.enter();
    let json = page.build()?;
    tracing::info!("{url}");
    Ok(Json(json))
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

pub fn serve() -> Result<(), anyhow::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let app = Router::new().fallback(get_json);

            let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    Ok(())
}
