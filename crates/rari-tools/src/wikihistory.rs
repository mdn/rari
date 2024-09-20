use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

use crate::error::ToolError;

pub fn update_wiki_history(locale: Locale, pairs: &Vec<(String, String)>) -> Result<(), ToolError> {
    // Construct the path to "_wikihistory.json"
    let locale_content_root = root_for_locale(locale)?;
    let wiki_history_path = Path::new(locale_content_root)
        .join(locale.as_folder_str())
        .join("_wikihistory.json");

    // Read the content of the JSON file
    let wiki_history_content = fs::read_to_string(&wiki_history_path)?;

    // Parse the JSON content into a BTreeMap (sorted map)
    let mut all: BTreeMap<String, Value> = serde_json::from_str(&wiki_history_content)?;

    for (old_slug, new_slug) in pairs {
        if all.contains_key(old_slug) {
            all.insert(new_slug.to_string(), all[old_slug].clone());
            all.remove(old_slug);
        }
    }
    // Serialize the sorted map back to pretty JSON
    let mut json_string = serde_json::to_string_pretty(&all)?;
    // Add a trailing newline
    json_string.push_str("\n");

    // Write the updated JSON back to the file
    fs::write(&wiki_history_path, json_string)?;

    Ok(())
}
