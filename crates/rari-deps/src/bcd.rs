use std::collections::HashMap;
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
        extract_spec_urls(&path)?;
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

#[cfg(test)]
mod test {
    use super::{ensure_spec_urls, update_bcd_data};
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
}
