use std::collections::HashMap;
use std::thread::{self, JoinHandle};

use crossbeam_channel::Receiver;
use pagefind::api::PagefindIndex;
use pagefind::options::PagefindServiceConfig;
use rari_doc::build::PagefindMessage;
use rari_doc::pages::json::BuiltPage;
use rari_types::locale::Locale;

use crate::error::ToolError;

pub fn spawn_pagefind_indexer(
    rx: Receiver<PagefindMessage>,
) -> JoinHandle<Result<HashMap<Locale, PagefindIndex>, ToolError>> {
    thread::spawn(move || {
        let locales = Locale::for_generic_and_spas();
        #[allow(unused_mut)]
        let mut indexes = locales
            .iter()
            .map(|locale| {
                let options = PagefindServiceConfig::builder()
                    .force_language(locale.as_iso_639_str().to_string())
                    .build();
                (*locale, PagefindIndex::new(Some(options)).unwrap())
            })
            .collect::<HashMap<Locale, PagefindIndex>>();

        for msg in rx {
            match msg {
                PagefindMessage::Doc(built_page) => {
                    let (locale, title) = match built_page {
                        BuiltPage::Doc(jsdoc) => (jsdoc.doc.locale, jsdoc.doc.title.clone()),
                        BuiltPage::Curriculum(jsdoc) => (jsdoc.doc.locale, jsdoc.doc.title.clone()),
                        BuiltPage::BlogPost(jsdoc) => (jsdoc.doc.locale, jsdoc.doc.title.clone()),
                        BuiltPage::ContributorSpotlight(jsdoc) => {
                            (Locale::EnUs, jsdoc.page_title.clone())
                        }
                        BuiltPage::GenericPage(jsdoc) => (Locale::EnUs, jsdoc.page_title.clone()),
                        BuiltPage::SPA(jsdoc) => (Locale::EnUs, jsdoc.page_title.to_string()),
                        BuiltPage::Home(jsdoc) => (Locale::EnUs, jsdoc.page_title.to_string()),
                    };
                    println!("XX {}: {}", locale, title)
                }
                PagefindMessage::Done => break,
            }
        }
        Ok(indexes)
    })
}
