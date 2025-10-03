use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use css_syntax_types::Css;
use rari_types::globals::deps;
use rari_utils::io::read_to_string;
use semver::VersionReq;
use serde_json::Value;

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
            if value["descriptors"].is_array() {
                let descriptors =
                    by_name(serde_json::from_value(value["descriptors"].take()).unwrap());
                value["descriptors"] = serde_json::to_value(descriptors).unwrap();
            }
            if value["values"].is_array() {
                let values = by_name(serde_json::from_value(value["values"].take()).unwrap());
                value["values"] = serde_json::to_value(values).unwrap();
            }
            (name, value)
        })
        .collect()
}

fn list_all(folder: &Path) -> Result<BTreeMap<String, Css>, DepsError> {
    let mut all = Vec::new();
    let files = fs::read_dir(folder.join("package"))?;

    for file in files {
        let file = file?;
        let path = file.path();
        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();

        if path.is_file() && path.extension().unwrap() == "json" && file_name != "package" {
            let text = read_to_string(&path)?;
            let json: Value = serde_json::from_str(&text)?;
            all.push((file_name, json));
        }
    }

    let parse: BTreeMap<String, Css> = all
        .into_iter()
        .map(|(name, mut data)| {
            data["properties"] = serde_json::to_value(by_name(
                serde_json::from_value(data["properties"].take()).unwrap(),
            ))
            .unwrap();
            data["selectors"] = serde_json::to_value(by_name(
                serde_json::from_value(data["selectors"].take()).unwrap(),
            ))
            .unwrap();
            data["atrules"] = serde_json::to_value(by_name(
                serde_json::from_value(data["atrules"].take()).unwrap(),
            ))
            .unwrap();
            data["values"] = serde_json::to_value(by_name(
                serde_json::from_value(data["values"].take()).unwrap(),
            ))
            .unwrap();

            if data["warnings"].is_array() {
                data["warnings"] = serde_json::to_value(by_name(
                    serde_json::from_value(data["warnings"].take()).unwrap(),
                ))
                .unwrap();
            }
            (name, serde_json::from_value(data).unwrap())
        })
        .collect();

    Ok(parse)
}

pub fn update_webref_css(base_path: &Path) -> Result<(), DepsError> {
    if let Some(package_path) = get_package(
        "@webref/css",
        &deps()
            .webref_css
            .clone()
            .or_else(|| Some(VersionReq::parse(">=6.0.0, <7.0.0").unwrap())),
        base_path,
    )? {
        let webref_css_dest_path = package_path.join("webref_css.json");
        let webref_css = list_all(&package_path)?;
        fs::write(webref_css_dest_path, serde_json::to_string(&webref_css)?)?;
    }
    Ok(())
}
