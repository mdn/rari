use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use serde_json::Value;

use crate::error::ToolError;

pub fn update_wiki_history(locale: Locale, pairs: &[(String, String)]) -> Result<(), ToolError> {
    let mut all = read_wiki_history(locale)?;
    for (old_slug, new_slug) in pairs {
        if let Some(to) = all.remove(old_slug) {
            all.insert(new_slug.to_string(), to);
        }
    }
    write_wiki_history(locale, all)?;
    Ok(())
}

pub fn delete_from_wiki_history(locale: Locale, slugs: &[String]) -> Result<(), ToolError> {
    let mut all = read_wiki_history(locale)?;
    for slug in slugs {
        all.remove(slug);
    }
    write_wiki_history(locale, all)?;
    Ok(())
}

fn write_wiki_history(locale: Locale, all: BTreeMap<String, Value>) -> Result<(), ToolError> {
    let wiki_history_path = wiki_history_path(locale)?;
    let file = File::create(&wiki_history_path)?;
    let mut buffer = BufWriter::new(file);
    // Write the updated pretty JSON back to the file
    serde_json::to_writer_pretty(&mut buffer, &all)?;
    // Add a trailing newline
    buffer.write_all(b"\n")?;
    Ok(())
}

fn read_wiki_history(locale: Locale) -> Result<BTreeMap<String, Value>, ToolError> {
    let wiki_history_path = wiki_history_path(locale)?;
    // Read the content of the JSON file
    let wiki_history_content = fs::read_to_string(&wiki_history_path)?;
    // Parse the JSON content into a BTreeMap (sorted map)
    let all: BTreeMap<String, Value> = serde_json::from_str(&wiki_history_content)?;
    Ok(all)
}

fn wiki_history_path(locale: Locale) -> Result<String, ToolError> {
    let locale_content_root = root_for_locale(locale)?;
    Ok(Path::new(locale_content_root)
        .join(locale.as_folder_str())
        .join("_wikihistory.json")
        .to_string_lossy()
        .to_string())
}

#[cfg(test)]
pub(crate) fn test_get_wiki_history(locale: Locale) -> BTreeMap<String, Value> {
    read_wiki_history(locale).expect("Could not read wiki history")
}
