use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use tower_http::trace::{self, TraceLayer};
use tracing::{error, info, Level};

use crate::embed::embeds;

#[derive(serde::Serialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(serde::Deserialize)]
struct EmbedRequest {
    texts: Vec<String>,
}

#[axum::debug_handler]
async fn post_embed_handler(Json(payload): Json<EmbedRequest>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let chars = payload.texts.join(" ").len();
    let tlen = payload.texts.len();

    match embeds(payload.texts) {
        Ok(embeddings) => {
            info!(
                "embed {} chars over {} texts in {}ms",
                chars,
                tlen,
                start.elapsed().as_millis()
            );
            (StatusCode::OK, Json(EmbedResponse { embeddings })).into_response()
        }
        Err(e) => {
            tracing::error!("error embeddings: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
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

pub fn serve() -> Result<(), anyhow::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let app = Router::new().route("/", post(post_embed_handler)).layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(trace::DefaultOnResponse::new().level(Level::ERROR)),
            );
            info!("listening on http://localhost:8084");
            let listener = tokio::net::TcpListener::bind("0.0.0.0:8084").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
    Ok(())
}
