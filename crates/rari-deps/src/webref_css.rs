use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use css_syntax_types::{BrowserSpec, Css, SpecLink};
use rari_types::globals::deps;
use rari_utils::io::read_to_string;
use serde_json::Value;
use url::Url;

use crate::error::DepsError;
use crate::npm::get_package;

fn normalize_name(name: &str) -> String {
    name.trim_start_matches('<')
        .trim_end_matches('>')
        .to_string()
}

fn by_name(values: Vec<Value>) -> BTreeMap<String, Value> {
    values
        .into_iter()
        .map(|mut value| {
            let name = normalize_name(value["name"].as_str().unwrap());

            // Recursively process nested descriptors
            if value["descriptors"].is_array() {
                let descriptors =
                    by_name(serde_json::from_value(value["descriptors"].take()).unwrap());
                value["descriptors"] = serde_json::to_value(descriptors).unwrap();
            }

            // Recursively process nested values (for compatibility with nested structures)
            if value["values"].is_array() {
                let values = by_name(serde_json::from_value(value["values"].take()).unwrap());
                value["values"] = serde_json::to_value(values).unwrap();
            }

            (name, value)
        })
        .collect()
}

// Add href_title fields based on browser specs mapping
fn enrich_with_specs(data: &mut Value, url_to_title: &BTreeMap<String, String>) {
    match data {
        Value::Object(obj) => {
            // If this object has an href or url field, create a specs field

            let url_field = obj.get("href").or_else(|| obj.get("url"));
            let field_name = if obj.contains_key("href") {
                "href"
            } else {
                "url"
            };

            if let Some(href_val) = url_field {
                if let Some(href_str) = href_val.as_str() {
                    if let Ok(url) = Url::parse(href_str) {
                        let mut url_without_fragment = url.clone();
                        url_without_fragment.set_fragment(None);
                        let title = url_to_title
                            .get(url_without_fragment.as_str())
                            .cloned()
                            .unwrap_or_else(|| "CSS Specification".to_string());

                        let spec = SpecLink { title, url };
                        obj.insert("specLink".to_string(), serde_json::to_value(spec).unwrap());
                    }
                }
                obj.remove(field_name);
            }

            // Recursively process all nested values, but skip spec_link fields to avoid infinite loops
            for (key, value) in obj.iter_mut() {
                if key != "specLink" {
                    enrich_with_specs(value, url_to_title);
                }
            }
        }
        Value::Array(arr) => {
            // Recursively process array elements
            for item in arr {
                enrich_with_specs(item, url_to_title);
            }
        }
        _ => {}
    }
}

fn transform(
    folder: &Path,
    url_to_title: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Css>, DepsError> {
    // Read the single css.json file (v7+ format)
    let css_json_path = folder.join("package").join("css.json");
    let text = read_to_string(&css_json_path)?;
    let mut data: Value = serde_json::from_str(&text)?;

    if data["properties"].is_array() {
        data["properties"] = serde_json::to_value(by_name(
            serde_json::from_value(data["properties"].take()).unwrap_or_default(),
        ))?;
    }

    if data["selectors"].is_array() {
        data["selectors"] = serde_json::to_value(by_name(
            serde_json::from_value(data["selectors"].take()).unwrap_or_default(),
        ))?;
    }

    if data["atrules"].is_array() {
        data["atrules"] = serde_json::to_value(by_name(
            serde_json::from_value(data["atrules"].take()).unwrap_or_default(),
        ))?;
    }

    if data["functions"].is_array() {
        data["functions"] = serde_json::to_value(by_name(
            serde_json::from_value(data["functions"].take()).unwrap_or_default(),
        ))?;
    }

    if data["types"].is_array() {
        data["types"] = serde_json::to_value(by_name(
            serde_json::from_value(data["types"].take()).unwrap_or_default(),
        ))?;
    }

    // Enrich all items with href_title fields
    enrich_with_specs(&mut data, url_to_title);

    // Create a single entry with "CSS" as the key for the combined data
    let mut result = BTreeMap::new();
    result.insert("CSS".to_string(), serde_json::from_value(data)?);
    Ok(result)
}

fn process_browser_specs(folder: &Path) -> Result<BTreeMap<String, String>, DepsError> {
    let index_json_path = folder.join("package").join("index.json");
    let text = read_to_string(&index_json_path)?;
    let specs: Vec<BrowserSpec> = serde_json::from_str(&text)?;

    let mut url_to_title = BTreeMap::new();

    for spec in specs {
        // Add the main spec URL
        url_to_title.insert(spec.url.clone(), spec.title.clone());

        // Also add nightly URL if available
        if let Some(nightly) = &spec.nightly {
            url_to_title.insert(nightly.url.clone(), spec.title.clone());
        }

        // Also add release URL if different from main URL
        if let Some(release) = &spec.release {
            if release.url != spec.url {
                url_to_title.insert(release.url.clone(), spec.title.clone());
            }
        }

        // Add series URLs if different
        if let Some(release_url) = spec.series.release_url {
            if release_url != spec.url {
                url_to_title.insert(release_url.clone(), spec.title.clone());
            }
        }

        if let Some(nightly_url) = &spec.series.nightly_url {
            if Some(nightly_url) != spec.nightly.as_ref().map(|n| &n.url) {
                url_to_title.insert(nightly_url.clone(), spec.title.clone());
            }
        }
    }

    Ok(url_to_title)
}

// Since version 7.x of webref-css, there is no link title information available (under `specs` key in the 6.x series)
// As a workaround, we use a link->title mapping extracted from the `browser-specs` package.
pub fn update_webref_css(base_path: &Path) -> Result<(), DepsError> {
    let url_to_title = if let Some(browser_specs_path) =
        get_package("browser-specs", &deps().browser_specs, base_path)?
    {
        let url_titles_dest_path = browser_specs_path.join("url-titles.json");
        let browser_specs = process_browser_specs(&browser_specs_path)?;

        // Write out the browser specs mapping
        fs::write(
            &url_titles_dest_path,
            serde_json::to_string(&browser_specs)?,
        )?;

        browser_specs
    } else {
        let json_path = base_path.join("browser-specs").join("url-titles.json");

        if let Ok(existing_browser_specs) = fs::read_to_string(json_path) {
            let browser_specs: BTreeMap<String, String> =
                serde_json::from_str(&existing_browser_specs)?;
            browser_specs
        } else {
            tracing::warn!(
                "could not read url-titles.json, link title fields will not be populated"
            );
            BTreeMap::new()
        }
    };

    // Process the CSS data with the URL-to-title map passed in.
    if let Some(package_path) = get_package("@webref/css", &deps().webref_css, base_path)? {
        let webref_css = transform(&package_path, &url_to_title)?;
        let webref_css_dest_path = package_path.join("webref_css.json");
        fs::write(webref_css_dest_path, serde_json::to_string(&webref_css)?)?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    fn url_titles_map(json_path: &PathBuf) -> BTreeMap<String, String> {
        if let Ok(existing_browser_specs) = fs::read_to_string(json_path) {
            let ret: BTreeMap<String, String> =
                serde_json::from_str(&existing_browser_specs).unwrap();
            ret
        } else {
            println!("could not read url-titles.json, link title fields will not be populated");
            BTreeMap::new()
        }
    }

    #[test]
    fn test_transform() {
        let url_to_title = url_titles_map(&PathBuf::from("test/url-titles.json"));
        // println!("url_to_title: {:#?}", url_to_title);
        let input_path = PathBuf::from("test");
        let _webref_css = transform(&input_path, &url_to_title).unwrap();
    }
}
