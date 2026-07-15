//! # Web Specifications Module
//!
//! The `specs` module provides functionality for managing and accessing specifications (Webspec, BCD)
//! related to web technologies. It includes utilities for loading specification data from files,
//! deduplicating specification URLs, and extracting relevant specifications based on queries.

use std::sync::LazyLock;

use rari_data::specs::{BCDSpecUrls, WebSpecs};
use rari_types::globals::{data_dir, json_spec_data_lookup};
use schemars::JsonSchema;
use serde::Serialize;
use tracing::error;

use crate::utils::deduplicate;

#[derive(Debug, Clone, Default)]
struct SpecsData {
    pub web_specs: WebSpecs,
    pub bcd_spec_urls: BCDSpecUrls,
}

static SPECS: LazyLock<SpecsData> = LazyLock::new(|| {
    let web_specs = WebSpecs::from_file(&data_dir().join("web-specs/package/index.json"))
        .map_err(|e| {
            error!("Failed to load web-specs data: {e:?}");
            e
        })
        .ok();
    let bcd_spec_urls =
        match BCDSpecUrls::from_file(&data_dir().join("@mdn/browser-compat-data/spec_urls.json")) {
            Ok(ok) => Some(ok),
            Err(e) => {
                error!("Failed to load BCD spec URLs: {e:?}");
                None
            }
        };
    match (web_specs, bcd_spec_urls) {
        (Some(web_specs), Some(bcd_spec_urls)) => SpecsData {
            web_specs,
            bcd_spec_urls,
        },
        _ => Default::default(),
    }
});

/// Represents a web technology specification.
///
/// The `Specification` struct is used to store information about a web spec,
/// including its title and the URL to its Browser Compatibility Data (BCD) specification.
///
/// # Fields
///
/// * `bcd_specification_url` - A `String` that holds the URL to the BCD specification.
/// * `title` - A `&'static str` that holds the title of the specification.
#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct Specification {
    #[serde(rename = "bcdSpecificationURL")]
    pub bcd_specification_url: String,
    pub title: &'static str,
}

/// Extracts relevant specifications based on the provided query and specification URLs.
///
/// This function searches for specifications that match the given query and specification URLs.
/// It first checks if the query is not empty and the specification URLs are empty. If so, it looks up
/// the BCD (Browser Compatibility Data) specification URLs that match the query keys. If no exact match is found,
/// it checks if the query is a prefix of any keys. The function then deduplicates the collected URLs and
/// maps them to `Specification` objects, which include the BCD specification URL and the title of the specification.
///
/// # Arguments
///
/// * `query` - A slice of strings that holds the query keys to search for specifications.
/// * `spec_urls` - A slice of strings that holds the specification URLs to be included.
///
/// # Returns
///
/// * `Vec<Specification>` - Returns a vector of `Specification` objects that match the query and specification URLs.
pub(crate) fn extract_specifications(query: &[String], spec_urls: &[String]) -> Vec<Specification> {
    let mut all_spec_urls: Vec<&String> = vec![];
    if !query.is_empty() && spec_urls.is_empty() {
        for q in query {
            if let Some(urls) = SPECS.bcd_spec_urls.specs_urls_by_key.get(q) {
                all_spec_urls.extend(urls.iter())
            } else {
                // no spec_urls found for the full key so we check if q is a prefix of some keys.
                // e.g. javascript.operators
                all_spec_urls.extend(
                    SPECS
                        .bcd_spec_urls
                        .specs_urls_by_key
                        .iter()
                        .filter_map(|(k, v)| if k.starts_with(q) { Some(v) } else { None })
                        .flatten(),
                )
            }
        }
    }
    all_spec_urls.extend(spec_urls.iter());
    all_spec_urls = deduplicate(all_spec_urls);
    all_spec_urls
        .into_iter()
        .map(|url| {
            let url_no_hash = &url[..url.find('#').unwrap_or(url.len())];
            if let Some(spec) = SPECS.web_specs.get_spec(url_no_hash) {
                Specification {
                    bcd_specification_url: url.to_string(),
                    title: spec.title.as_str(),
                }
            } else {
                match json_spec_data_lookup().get(url_no_hash) {
                    Some(title) => Specification {
                        bcd_specification_url: url.to_string(),
                        title: title.as_str(),
                    },
                    None => Specification {
                        bcd_specification_url: url.to_string(),
                        title: "Unknown specification",
                    },
                }
            }
        })
        .collect()
}
