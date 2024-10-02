use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use serde_json::Value;

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
        if let Some(to) = all.remove(old_slug) {
            all.insert(new_slug.to_string(), to);
        }
    }

    let file = File::create(&wiki_history_path)?;
    let mut buffer = BufWriter::new(file);
    // Write the updated pretty JSON back to the file
    serde_json::to_writer_pretty(&mut buffer, &all)?;
    // Add a trailing newline
    buffer.write_all(b"\n")?;

    Ok(())
}
