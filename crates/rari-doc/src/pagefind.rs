use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use crossbeam_channel::Receiver;
use pagefind::api::PagefindIndex;
use pagefind::options::PagefindServiceConfig;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::build::PagefindIndexMessage;
use crate::error::DocError;
use crate::pages::json::{BuiltPage, JsonBlogPostPage, JsonCurriculumPage, JsonDocPage};

pub fn spawn_pagefind_indexer(
    rx: Receiver<PagefindIndexMessage>,
) -> JoinHandle<Result<HashMap<Locale, PagefindIndex>, DocError>> {
    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let locales = Locale::for_generic_and_spas();
                let mut indexes = locales
                    .iter()
                    .map(|locale| {
                        let options = PagefindServiceConfig::builder()
                            .force_language(locale.as_iso_639_str().to_string())
                            .build();
                        (*locale, PagefindIndex::new(Some(options)).unwrap())
                    })
                    .collect::<HashMap<Locale, PagefindIndex>>();
                let mut count = 0;
                for msg in rx {
                    match msg {
                        PagefindIndexMessage::Doc(locale, built_page) => {
                            let data = match built_page {
                                BuiltPage::Doc(jsdoc) => {
                                    Some((jsdoc.url.clone(), doc_html_for_search_index(&jsdoc)))
                                }
                                BuiltPage::Curriculum(jsdoc) => Some((
                                    jsdoc.url.clone(),
                                    curriculum_html_for_search_index(&jsdoc),
                                )),
                                BuiltPage::BlogPost(jsdoc) => {
                                    Some((jsdoc.url.clone(), blog_html_for_search_index(&jsdoc)))
                                }
                                _ => None,
                            };
                            if let Some((url, html)) = data {
                                // println!("\n\nIndexing {}\n{}", url, html);
                                let index = indexes.get_mut(&locale).unwrap();
                                index
                                    .add_html_file(None, Some(url.to_string()), html)
                                    .await
                                    .unwrap();
                                count += 1;
                            }
                        }
                        PagefindIndexMessage::Done => break,
                    }
                }
                println!("Indexed {} pages", count);

                for (locale, index) in &mut indexes {
                    let mut path = PathBuf::from("/tmp");
                    path.push(locale.as_folder_str());
                    println!(
                        "Writing index files for {} to {}",
                        locale,
                        path.to_string_lossy()
                    );
                    fs::create_dir_all(&path).unwrap();
                    index
                        .write_files(Some(path.to_string_lossy().to_string()))
                        .await
                        .unwrap();
                }

                Ok(indexes)
            })
    })
}

fn doc_html_for_search_index(doc: &JsonDocPage) -> String {
    let mut html = String::new();
    html.push_str(&concat_strs!(
        "<html><head><title>",
        &doc.doc.title,
        "</title></head><body>"
    ));
    for section in &doc.doc.body {
        if let crate::pages::json::Section::Prose(prose) = section {
            if let Some(title) = &prose.title {
                let tag = if prose.is_h3 { "h3" } else { "h2" };
                let id = prose.id.as_deref().unwrap_or_default();
                html.push_str(&concat_strs!(
                    "\n<", tag, " id=\"", id, "\">", &title, "</", tag, ">"
                ));
            }
            html.push_str(&prose.content);
        }
    }
    html.push_str("\n</body></html>");
    html
}
fn curriculum_html_for_search_index(doc: &JsonCurriculumPage) -> String {
    let mut html = String::new();
    html.push_str(&doc.doc.title);
    html
}
fn blog_html_for_search_index(doc: &JsonBlogPostPage) -> String {
    let mut html = String::new();
    html.push_str(&doc.doc.title);
    html
}
