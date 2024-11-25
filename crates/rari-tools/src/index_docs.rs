use html2text::from_read;
use meilisearch_sdk::client::Client;
use rari_doc::pages::json::{BuiltPage, Section};
use rari_doc::pages::page::{Page, PageBuilder};
use rari_types::locale::Locale;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh64::xxh64;

use crate::error::ToolError;
use crate::utils::read_all_doc_pages;

#[derive(Serialize, Deserialize, Debug)]
struct MDoc {
    id: String,
    title: String,
    body: String,
    url: String,
}

const MEILISEARCH_URL: &str = "http://localhost:7700";
const MEILISEARCH_API_KEY: &str = "dFdgYlqSf3TaQRqgMC35PAQFclolikAJo6V5-jI4xeE";
// const MEILISEARCH_INDEX: &str = "mdn";
const MEILISEARCH_INDEX: &str = "mdn2";

pub fn index_meili_docs() -> Result<(), ToolError> {
    let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY)).unwrap();

    let index = client.index(MEILISEARCH_INDEX);

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let mdocs = read_all_doc_pages()
                .unwrap()
                .into_par_iter()
                .filter(|((locale, _), page)| {
                    *locale == Locale::EnUs && matches!(page, Page::Doc(_))
                })
                // .take(100)
                .filter_map(|((_, _path), page)| {
                    let built_page = page.build().unwrap();
                    match &built_page {
                        BuiltPage::Doc(d) => {
                            let doc = d.doc.clone();
                            let body = doc.body;
                            let mdocs = body
                                .into_iter()
                                .filter_map(|section| match section {
                                    Section::Prose(prose) => {
                                        let body =
                                            from_read(std::io::Cursor::new(&prose.content), 80)
                                                .unwrap_or_default();
                                        if body.is_empty() {
                                            return None;
                                        }
                                        let title = format!(
                                            "{} - {}",
                                            doc.title,
                                            prose.title.unwrap_or_default()
                                        );
                                        println!("title: {}", title);
                                        Some(MDoc {
                                            id: hash_string(&title),
                                            title,
                                            body,
                                            url: d.url.clone(),
                                        })
                                    }
                                    _ => None,
                                })
                                .collect::<Vec<_>>();
                            Some(mdocs)
                        }
                        _ => None,
                    }
                })
                .flatten()
                .collect::<Vec<_>>();
            // println!(
            //     "mdocs: {}",
            //     mdocs
            //         .iter()
            //         .map(|mdoc| format!("{}\n\n{}", mdoc.title, mdoc.url))
            //         .collect::<Vec<_>>()
            //         .join("\n")
            // );
            index
                .add_documents_in_batches(&mdocs, Some(500), Some("id"))
                .await
                .unwrap();
        });

    Ok(())
}

fn hash_string(input: &str) -> String {
    let hash = xxh64(input.as_bytes(), 123);
    format!("{:016x}", hash)
}
