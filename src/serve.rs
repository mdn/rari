use anyhow::Error;
use rari_doc::docs::build::{build_blog_post, build_curriculum, build_doc, build_dummy};
use rari_doc::docs::json::BuiltDocy;
use rari_doc::docs::page::{Page, PageLike};
use serde_json::Value;
use tiny_http::{Response, Server};
use tracing::{error, span, Level};

fn get_json(url: &str) -> Result<BuiltDocy, Error> {
    let page = Page::page_from_url_path(url)?;

    let slug = &page.slug();
    let locale = page.locale();
    let span = span!(Level::ERROR, "page", "{}:{}", locale, slug);
    let _enter = span.enter();
    let json = match page {
        Page::Doc(doc) => build_doc(&doc)?,
        Page::BlogPost(post) => build_blog_post(&post)?,
        Page::Dummy(dummy) => build_dummy(&dummy)?,
        Page::Curriculum(curriculim) => build_curriculum(&curriculim)?,
    };
    Ok(json)
}

pub fn serve() -> Result<(), Error> {
    let server = Server::http("0.0.0.0:8083").unwrap();

    for request in server.incoming_requests() {
        let url = request.url();
        let url_span = span!(Level::ERROR, "url", "{}", url);
        let _url_enter = url_span.enter();
        match get_json(url) {
            Ok(out) => {
                let data = serde_json::to_string(&out).unwrap();

                request.respond(
                    Response::from_data(data.as_bytes()).with_header(
                        "Content-Type: application/json; charset=utf-8"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                )?;
            }
            Err(e) => {
                error!("{e}");
                request.respond(
                    Response::from_data(
                        serde_json::to_string_pretty(&Value::Null)
                            .unwrap()
                            .as_bytes(),
                    )
                    .with_header(
                        "Content-Type: application/json; charset=utf-8"
                            .parse::<tiny_http::Header>()
                            .unwrap(),
                    ),
                )?;
            }
        }
    }
    Ok(())
}
