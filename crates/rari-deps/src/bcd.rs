use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use rari_types::globals::deps;
use rari_utils::io::read_to_string;
use serde_json::Value;

use crate::error::DepsError;
use crate::npm::get_package;

pub fn update_bcd(base_path: &Path) -> Result<(), DepsError> {
    update_bcd_data(base_path, |base_path| {
        get_package("@mdn/browser-compat-data", &deps().bcd, base_path)
    })?;
    get_package("web-specs", &deps().web_specs, base_path)?;
    Ok(())
}

fn update_bcd_data(
    base_path: &Path,
    get_bcd_package: impl FnOnce(&Path) -> Result<Option<PathBuf>, DepsError>,
) -> Result<(), DepsError> {
    if let Some(path) = get_bcd_package(base_path)? {
        extract_data(&path)?;
    } else {
        ensure_spec_urls(&base_path.join("@mdn/browser-compat-data"))?;
    }
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

/// Regenerates `spec_urls.json` if it is missing or truncated, e.g. after an interrupted update.
fn ensure_spec_urls(package_path: &Path) -> Result<(), DepsError> {
    let output_path = package_path.join("spec_urls.json");
    let valid = read_to_string(&output_path)
        .ok()
        .and_then(|data| serde_json::from_str::<HashMap<String, Vec<String>>>(&data).ok())
        .is_some();
    if !valid {
        extract_spec_urls(package_path)?;
    }
    Ok(())
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
    extract_spec_urls(package_path)?;
    let text = read_to_string(package_path.join("package/data.json"))?;
    let json: Value = serde_json::from_str(&text)?;
    let mut keys = HashSet::new();
    gather_bcd_keys(&json, "", &mut keys);
    fs::write(package_path.join("bcd_keys.json"), serde_json::to_string(&keys)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::{ensure_spec_urls, gather_bcd_keys, update_bcd_data};
    use serde_json::json;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn write_data_json(package_path: &Path, spec_url: &str) {
        fs::create_dir_all(package_path.join("package")).unwrap();
        fs::write(
            package_path.join("package/data.json"),
            json!({"css": {"types": {"color": {"__compat": {"spec_url": spec_url}}}}}).to_string(),
        )
        .unwrap();
    }

    #[test]
    fn test_ensure_spec_urls() {
        struct Case {
            name: &'static str,
            output: Option<&'static str>,
            unchanged: bool,
        }

        const VALID: &str = "{\n  \"css.types.color\": [\"url\"]\n}";

        let cases = [
            Case {
                name: "missing",
                output: None,
                unchanged: false,
            },
            Case {
                name: "invalid",
                output: Some("not json"),
                unchanged: false,
            },
            Case {
                name: "valid",
                output: Some(VALID),
                unchanged: true,
            },
        ];

        for Case {
            name,
            output,
            unchanged,
        } in cases
        {
            let dir = tempdir().unwrap();
            let package_path = dir.path();
            write_data_json(package_path, "url");
            if let Some(output) = output {
                fs::write(package_path.join("spec_urls.json"), output).unwrap();
            }

            ensure_spec_urls(package_path).unwrap();

            let actual = fs::read_to_string(package_path.join("spec_urls.json")).unwrap();
            if unchanged {
                assert_eq!(actual, VALID, "{name}");
            } else {
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&actual).unwrap(),
                    json!({"css.types.color": ["url"]}),
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn test_update_bcd_data() {
        struct Case {
            name: &'static str,
            downloaded: bool,
            output: Option<&'static str>,
        }

        let cases = [
            Case {
                name: "downloaded package replaces valid spec URLs",
                downloaded: true,
                output: Some(r#"{"css.types.color": ["old-url"]}"#),
            },
            Case {
                name: "cached package regenerates missing spec URLs",
                downloaded: false,
                output: None,
            },
        ];

        for Case {
            name,
            downloaded,
            output,
        } in cases
        {
            let dir = tempdir().unwrap();
            let package_path = dir.path().join("@mdn/browser-compat-data");
            write_data_json(&package_path, "new-url");
            if let Some(output) = output {
                fs::write(package_path.join("spec_urls.json"), output).unwrap();
            }

            update_bcd_data(dir.path(), |_| Ok(downloaded.then(|| package_path.clone()))).unwrap();

            let actual = fs::read_to_string(package_path.join("spec_urls.json")).unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&actual).unwrap(),
                json!({"css.types.color": ["new-url"]}),
                "{name}"
            );
        }
    }

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
