use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use css_syntax_types::{BrowserSpec, SpecLink, WebrefCss};
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

fn by_for_and_name(values: Vec<Value>) -> BTreeMap<String, BTreeMap<String, Value>> {
    let mut result: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();

    for mut value in values {
        let name = normalize_name(value["name"].as_str().unwrap());

        // Recursively process nested descriptors
        if value["descriptors"].is_array() {
            let descriptors = by_name(serde_json::from_value(value["descriptors"].take()).unwrap());
            value["descriptors"] = serde_json::to_value(descriptors).unwrap();
        }

        // Recursively process nested values (for compatibility with nested structures)
        if value["values"].is_array() {
            let nested_values = by_name(serde_json::from_value(value["values"].take()).unwrap());
            value["values"] = serde_json::to_value(nested_values).unwrap();
        }

        // Handle 'for' key - could be a string, array, or missing
        // Add the entry to the global scope as well, not all pages have the needed
        // browser-compat key to properly find the entry in its scope.
        let for_keys: Vec<String> = match &value["for"] {
            Value::String(s) => vec![normalize_name(s), "__global_scope__".to_string()],
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(normalize_name))
                .chain(vec!["__global_scope__".to_string()])
                .collect(),
            _ => vec!["__global_scope__".to_string()],
        };

        // Insert the value into each 'for' group
        for for_key in for_keys {
            result
                .entry(for_key)
                .or_default()
                .insert(name.clone(), value.clone());
        }
    }

    result
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
                if let Some(href_str) = href_val.as_str()
                    && let Ok(url) = Url::parse(href_str)
                {
                    let mut url_without_fragment = url.clone();
                    url_without_fragment.set_fragment(None);
                    let title = url_to_title
                        .get(url_without_fragment.as_str())
                        .cloned()
                        .unwrap_or_else(|| "CSS Specification".to_string());

                    let spec = SpecLink {
                        title,
                        url: url_without_fragment,
                    };
                    obj.insert("specLink".to_string(), serde_json::to_value(spec).unwrap());
                }
                obj.remove(field_name);
            }

            if let Some(extended) = obj.get("extended")
                && let Some(extended) = extended.as_array()
            {
                let specs = extended
                    .iter()
                    .filter_map(|value| {
                        if let Some(href_str) = value.as_str()
                            && let Ok(mut url) = Url::parse(href_str)
                        {
                            let title = url_to_title
                                .get(value.as_str().unwrap())
                                .cloned()
                                .unwrap_or_else(|| "CSS Specification".to_string());
                            url.set_fragment(None);
                            // println!("Parsed URL: {} {}", title, url.as_str());
                            Some(SpecLink { title, url })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !specs.is_empty() {
                    obj.insert(
                        "extendedSpecLinks".to_string(),
                        serde_json::to_value(specs).unwrap(),
                    );
                }
                obj.remove("extended");
            }

            // Recursively process all nested values, but skip specLink and extendedSpecLinks fields to avoid infinite loops
            for (key, value) in obj.iter_mut() {
                if key != "specLink" && key != "extendedSpecLinks" {
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
) -> Result<WebrefCss, DepsError> {
    // Read the single css.json file (v7+ format)
    let css_json_path = folder.join("package").join("css.json");
    let text = read_to_string(&css_json_path)?;
    let mut data: Value = serde_json::from_str(&text)?;

    if data["properties"].is_array() {
        data["properties"] = serde_json::to_value(by_for_and_name(
            serde_json::from_value(data["properties"].take()).unwrap_or_default(),
        ))?;
    } else {
        return Err(DepsError::WebRefParseError(
            "Webref-CSS data lacks the `properties` array".to_string(),
        ));
    }

    if data["selectors"].is_array() {
        data["selectors"] = serde_json::to_value(by_for_and_name(
            serde_json::from_value(data["selectors"].take()).unwrap_or_default(),
        ))?;
    } else {
        return Err(DepsError::WebRefParseError(
            "Webref-CSS data lacks the `selectors` array".to_string(),
        ));
    }

    if data["atrules"].is_array() {
        data["atrules"] = serde_json::to_value(by_for_and_name(
            serde_json::from_value(data["atrules"].take()).unwrap_or_default(),
        ))?;
    } else {
        return Err(DepsError::WebRefParseError(
            "Webref-CSS data lacks the `atrules` array".to_string(),
        ));
    }

    if data["functions"].is_array() {
        data["functions"] = serde_json::to_value(by_for_and_name(
            serde_json::from_value(data["functions"].take()).unwrap_or_default(),
        ))?;
    } else {
        return Err(DepsError::WebRefParseError(
            "Webref-CSS data lacks the `functions` array".to_string(),
        ));
    }

    if data["types"].is_array() {
        data["types"] = serde_json::to_value(by_for_and_name(
            serde_json::from_value(data["types"].take()).unwrap_or_default(),
        ))?;
    } else {
        return Err(DepsError::WebRefParseError(
            "Webref-CSS data lacks the `types` array".to_string(),
        ));
    }

    // Enrich all items with href_title fields
    enrich_with_specs(&mut data, url_to_title);

    let result = serde_json::from_value(data)?;
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
        if let Some(release) = &spec.release
            && release.url != spec.url
        {
            url_to_title.insert(release.url.clone(), spec.title.clone());
        }

        // Add series URLs if different
        if let Some(release_url) = spec.series.release_url
            && release_url != spec.url
        {
            url_to_title.insert(release_url.clone(), spec.title.clone());
        }

        if let Some(nightly_url) = &spec.series.nightly_url
            && Some(nightly_url) != spec.nightly.as_ref().map(|n| &n.url)
        {
            url_to_title.insert(nightly_url.clone(), spec.title.clone());
        }
    }

    Ok(url_to_title)
}

/**
 * Returns a Map of URL strings to titles, by looking up the titles by the spec URL
 * in the browser-specs package.
 */
fn spec_url_to_title_map(base_path: &Path) -> Result<BTreeMap<String, String>, DepsError> {
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
    Ok(url_to_title)
}

// Since version 7.x of webref-css, there is no link title information available (under `specs` key in the 6.x series)
// As a workaround, we use a link->title mapping extracted from the `browser-specs` package.
pub fn update_webref_css(base_path: &Path) -> Result<(), DepsError> {
    if let Some(package_path) = get_package("@webref/css", &deps().webref_css, base_path)? {
        let url_to_title = spec_url_to_title_map(base_path)?;
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
        let input_path = PathBuf::from("test");
        let webref_css = transform(&input_path, &url_to_title).unwrap();
        // println!("webref_css: {:#?}", webref_css.atrules);
        assert!(!webref_css.atrules.is_empty());
        assert!(!webref_css.functions.is_empty());
        assert!(!webref_css.properties.is_empty());
        assert!(!webref_css.selectors.is_empty());
        assert!(!webref_css.types.is_empty());

        assert!(webref_css.atrules.contains_key("__global_scope__"));
        assert!(webref_css.functions.contains_key("__global_scope__"));
        assert!(webref_css.properties.contains_key("__global_scope__"));
        assert!(webref_css.selectors.contains_key("__global_scope__"));
        assert!(webref_css.types.contains_key("__global_scope__"));

        // The `align-self` property has some extended spec links, check that.
        assert!(
            webref_css
                .properties
                .get("__global_scope__")
                .unwrap()
                .contains_key("align-self")
        );
        let align_self = webref_css
            .properties
            .get("__global_scope__")
            .unwrap()
            .get("align-self")
            .unwrap();

        assert!(!align_self.extended_spec_links.is_empty());
    }
}
