use anyhow::Error;
use rari_doc::docs::build::{build_blog_post, build_curriculum, build_doc, build_dummy};
use rari_doc::docs::json::BuiltDocy;
use rari_doc::docs::page::Page;
use serde_json::Value;
use tiny_http::{Response, Server};

fn get_json(url: &str) -> Result<BuiltDocy, Error> {
    let doc = Page::page_from_url_path(url)?;

    let json = match doc {
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
        match get_json(request.url()) {
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
                println!("{e}");
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
