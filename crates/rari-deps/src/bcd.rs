use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rari_types::globals::deps;
use rari_utils::io::read_to_string;
use serde_json::Value;

use crate::error::DepsError;
use crate::npm::get_package;

pub fn update_bcd(base_path: &Path) -> Result<(), DepsError> {
    if let Some(path) = get_package("@mdn/browser-compat-data", &deps().bcd, base_path)? {
        extract_spec_urls(&path)?;
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

pub fn extract_spec_urls(package_path: &Path) -> Result<(), DepsError> {
    let text = read_to_string(package_path.join("package/data.json"))?;
    let json: Value = serde_json::from_str(&text)?;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    gather_spec_urls(&json, "", &mut map);
    let spec_urls_out_path = package_path.join("spec_urls.json");
    fs::write(spec_urls_out_path, serde_json::to_string(&map)?)?;
    Ok(())
}
