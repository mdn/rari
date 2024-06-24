use once_cell::sync::Lazy;
use rari_data::specs::{BCDSpecUrls, WebSpecs};
use rari_types::globals::{data_dir, json_spec_data_lookup};
use serde::Serialize;
use tracing::warn;

use crate::utils::deduplicate;

#[derive(Debug, Clone, Default)]
pub struct SpecsData {
    pub web_specs: WebSpecs,
    pub bcd_spec_urls: BCDSpecUrls,
}

pub static SPECS: Lazy<SpecsData> = Lazy::new(|| {
    let web_specs = WebSpecs::from_file(&data_dir().join("web-specs/package/index.json"))
        .map_err(|e| {
            warn!("{e:?}");
            e
        })
        .ok();
    let bcd_spec_urls =
        match BCDSpecUrls::from_file(&data_dir().join("@mdn/browser-compat-data/spec_urls.json")) {
            Ok(ok) => Some(ok),
            Err(e) => {
                warn!("{e:?}");
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

#[derive(Debug, Clone, Serialize, Default)]
pub struct Specification {
    #[serde(rename = "bcdSpecificationURL")]
    pub bcd_specification_url: String,
    pub title: &'static str,
}

pub fn extract_specifications(query: &[String], spec_urls: &[String]) -> Vec<Specification> {
    let mut all_spec_urls: Vec<&String> = vec![];
    if !query.is_empty() && spec_urls.is_empty() {
        for q in query {
            if let Some(urls) = SPECS.bcd_spec_urls.specs_urls_by_key.get(q) {
                all_spec_urls.extend(urls.iter())
            } else {
                // no spec_urls found for the full key os we check if q is a prefix of some keys.
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
