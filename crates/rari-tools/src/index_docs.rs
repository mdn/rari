use meilisearch_sdk::client::Client;
use rari_doc::pages::page::PageLike;
use rari_types::locale::Locale;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh64::xxh64;

use crate::error::ToolError;
use crate::utils::read_all_doc_pages;

#[derive(Serialize, Deserialize, Debug)]
struct MDoc {
    id: String,
    title: String,
    body: String,
}

const MEILISEARCH_URL: &str = "http://localhost:7700";
const MEILISEARCH_API_KEY: &str = "dFdgYlqSf3TaQRqgMC35PAQFclolikAJo6V5-jI4xeE";

pub fn index_meili_docs() -> Result<(), ToolError> {
    let client = Client::new(MEILISEARCH_URL, Some(MEILISEARCH_API_KEY)).unwrap();

    let index = client.index("mdn");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let mdocs = read_all_doc_pages()
                .unwrap()
                .into_iter()
                .filter(|((locale, _), _)| *locale == Locale::EnUs)
                // .take(5000)
                .map(|((_, path), page)| {
                    let body = page.content().to_string();
                    let title = page.title().to_string();
                    let id = hash_string(&path);
                    let doc = MDoc { id, title, body };
                    println!("Indexing: {}", doc.title);
                    doc
                })
                .collect::<Vec<_>>();
            index
                .add_documents_in_batches(&mdocs, Some(10), Some("id"))
                .await
                .unwrap();
        });

    Ok(())
}

fn hash_string(input: &str) -> String {
    let hash = xxh64(input.as_bytes(), 123);
    format!("{:016x}", hash)
}
