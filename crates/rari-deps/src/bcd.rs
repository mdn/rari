use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use rari_types::globals::deps;
use rari_utils::io::read_to_string;
use serde_json::Value;

use crate::error::DepsError;
use crate::npm::get_package;

pub fn update_bcd(base_path: &Path) -> Result<(), DepsError> {
    if let Some(path) = get_package("@mdn/browser-compat-data", &deps().bcd, base_path)? {
        extract_data(&path)?;
    }
    get_package("web-specs", &deps().web_specs, base_path)?;
    Ok(())
}

pub fn gather_spec_urls(value: &Value, path: &str, map: &mut HashMap<String, Vec<String>>) {
    match &value["__compat"]["spec_url"] {
        Value::String(spec_url) => {
            map.insert(path.to_string(), vec![spec_url.clone()]);
        }
        Value::Array(spec_urls) => {
            map.insert(
                path.to_string(),
                spec_urls
                    .iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect(),
            );
        }
        _ => {}
    };
    if let Value::Object(o) = value {
        for (k, v) in o.iter().filter(|(k, _)| *k != "__compat") {
            gather_spec_urls(
                v,
                &format!("{path}{}{k}", if path.is_empty() { "" } else { "." }),
                map,
            )
        }
    }
}

fn gather_bcd_keys(value: &Value, path: &str, keys: &mut HashSet<String>) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let mut has_compat = object.contains_key("__compat");
    for (key, value) in object.iter().filter(|(key, _)| *key != "__compat") {
        let child = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        has_compat |= gather_bcd_keys(value, &child, keys);
    }
    if has_compat && !path.is_empty() {
        keys.insert(path.to_string());
    }
    has_compat
}

fn extract_data(package_path: &Path) -> Result<(), DepsError> {
    let text = read_to_string(package_path.join("package/data.json"))?;
    let json: Value = serde_json::from_str(&text)?;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    gather_spec_urls(&json, "", &mut map);
    let spec_urls_out_path = package_path.join("spec_urls.json");
    fs::write(spec_urls_out_path, serde_json::to_string(&map)?)?;
    let mut keys = HashSet::new();
    gather_bcd_keys(&json, "", &mut keys);
    let bcd_keys_out_path = package_path.join("bcd_keys.json");
    fs::write(bcd_keys_out_path, serde_json::to_string(&keys)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::gather_bcd_keys;
    use serde_json::json;
    use std::collections::HashSet;

    #[test]
    fn test_gather_bcd_keys() {
        let data = json!({
            "css": {"types": {"color": {
                "color-mix": {"__compat": {"support": {}}},
                "color": {"display-p3": {"__compat": {"support": {}}}}
            }}},
            "browsers": {"firefox": {"name": "Firefox"}}
        });
        let mut keys = HashSet::new();

        gather_bcd_keys(&data, "", &mut keys);

        assert_eq!(
            keys,
            HashSet::from([
                "css".to_string(),
                "css.types".to_string(),
                "css.types.color".to_string(),
                "css.types.color.color-mix".to_string(),
                "css.types.color.color".to_string(),
                "css.types.color.color.display-p3".to_string(),
            ])
        );
    }
}
