use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use crossbeam_channel::Receiver;
use pagefind::api::PagefindIndex;
use pagefind::options::PagefindServiceConfig;
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use regex::Regex;

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
    // let level = (doc.url.split('/').count() - 3) as f32;
    // 7.0 is the default weight of h1, add a bit to it to make more top-level objects weigh higher
    // let additional_weight = (7.0 + (7.0 / level)).to_string();
    // let additional_weight = "9.0";
    // let additional_weight =
    //     if doc.url == "/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array" {
    //         "10.0"
    //     } else {
    //         "1.0"
    //     };
    let tag_title = Regex::new(r"<(\w*)>").unwrap();
    // let special_encoded_title = if tag_title.is_match(doc.doc.title.as_str()) {
    //     println!(
    //         "Special title: {} -> {}",
    //         doc.doc.title,
    //         tag_title.replace_all(doc.doc.title.as_str(), " oabr${1}cabr ")
    //     );
    //     std::borrow::Cow::Owned(
    //         tag_title
    //             .replace_all(doc.doc.title.as_str(), " oabr${1}cabr ")
    //             .to_string(),
    //     )
    // } else {
    //     std::borrow::Cow::Borrowed("")
    // };
    html.push_str(&concat_strs!(
        "<html><head><title>",
        &html_escape::encode_text(doc.doc.title.as_str()),
        "</title></head><body>",
        "<h1 data-pagefind-weight=\"9.0\">",
        &html_escape::encode_text(doc.doc.title.as_str()),
        "</h1>\n"
    ));
    for section in &doc.doc.body {
        if let crate::pages::json::Section::Prose(prose) = section {
            if let Some(title) = &prose.title {
                let tag = if prose.is_h3 { "h3" } else { "h2" };
                let id = prose.id.as_deref().unwrap_or_default();
                html.push_str(&concat_strs!(
                    "\n<",
                    tag,
                    " data-pagefind-weight=\"2.0\" id=\"",
                    id,
                    "\">",
                    &html_escape::encode_text(title),
                    "</",
                    tag,
                    ">"
                ));
            }
            html.push_str(&prose.content);
        }
    }
    html.push_str("\n</body>\n</html>");
    // // parse with kuchikiki and set weight
    // let document = parse_html().one(html);
    // for tag in document.select("h2").unwrap() {
    //     let node_ref = tag.as_node();
    //     if let Some(element) = node_ref.as_element() {
    //         element
    //             .attributes
    //             .borrow_mut()
    //             .insert("data-pagefind-weight", "2.0".to_string());
    //     }
    // }

    // if doc.url == "/en-US/docs/Web/HTML/Element/table" {
    //     println!("{}: {} \n\n", doc.url, html);
    // }

    // document.to_string()
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
