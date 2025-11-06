use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveTime, Utc};
use rari_types::locale::default_locale;
use rari_utils::concat_strs;
use rss::{Channel, Enclosure, Guid, Image, Item};

use crate::error::DocError;
use crate::pages::page::PageLike;
use crate::pages::types::blog::BlogPost;

fn guess_mime(file: &str) -> &str {
    match file.rsplit_once(".") {
        Some((_, "jpeg" | "jpg")) => "image/jpeg",
        Some((_, "png")) => "image/png",
        Some((_, "avif")) => "image/avif",
        Some((_, "jxl")) => "image/jxl",
        Some((_, "webp")) => "image/webp",
        _ => "",
    }
}

fn page_to_rss_item(post: &BlogPost, base_url: &str) -> Item {
    let link = concat_strs!(base_url, post.url());
    let guid = Guid {
        value: link.clone(),
        ..Default::default()
    };
    let image = concat_strs!(base_url, post.url(), &post.meta.image.file);
    let mime_type = guess_mime(&post.meta.image.file);
    let enclosure = Enclosure {
        url: image,
        length: "0".to_string(),
        mime_type: mime_type.to_owned(),
    };
    Item {
        title: Some(post.title().to_owned()),
        link: Some(link),
        description: Some(post.meta.description.clone()),
        author: Some(post.meta.author.clone()),
        guid: Some(guid),
        pub_date: Some(
            post.meta
                .date
                .and_time(NaiveTime::default())
                .and_utc()
                .to_rfc2822(),
        ),
        enclosure: Some(enclosure),
        ..Default::default()
    }
}

pub fn create_rss(
    path: &Path,
    documents: &[Arc<BlogPost>],
    base_url: &str,
) -> Result<(), DocError> {
    let title = "MDN Blog";
    let description = "The MDN Web Docs blog publishes articles about web development, open source software, web platform updates, tutorials, changes and updates to MDN, and more.";
    let link = concat_strs!(base_url, "/", default_locale().as_url_str(), "/blog/");
    let language = "en";
    let image = Image {
        url: concat_strs!(base_url, "/mdn-social-share.png"),
        title: title.to_owned(),
        link: link.clone(),
        width: None,
        height: None,
        description: None,
    };
    let copyright = "All rights reserved 2023, MDN";
    let docs = "https://validator.w3.org/feed/docs/rss2.html";

    let items: Vec<Item> = documents
        .iter()
        .map(|post| page_to_rss_item(post, base_url))
        .collect();

    let channel = Channel {
        title: title.to_owned(),
        link,
        description: description.to_owned(),
        language: Some(language.to_owned()),
        image: Some(image),
        copyright: Some(copyright.to_owned()),
        items,
        last_build_date: Some(Utc::now().to_rfc2822()),
        docs: Some(docs.to_owned()),
        ..Default::default()
    };

    let rss_string = channel.to_string();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?
    }

    let mut rss_file = fs::File::create(path)?;
    rss_file.write_all(&rss_string.into_bytes())?;
    rss_file.write_all(b"\n")?;

    Ok(())
}
