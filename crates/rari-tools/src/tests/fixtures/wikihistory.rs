use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, SecondsFormat};
use fake::Fake;
use fake::faker::chrono::en::DateTimeBetween;
use fake::faker::internet::en::Username;
use rari_doc::utils::root_for_locale;
use rari_types::locale::Locale;
use serde_json::Value;

#[allow(dead_code)]
pub(crate) struct WikihistoryFixtures {
    path: PathBuf,
    do_not_remove: bool,
}

impl WikihistoryFixtures {
    pub fn new(slugs: &Vec<String>, locale: Locale) -> Self {
        Self::new_internal(slugs, locale, false)
    }
    #[allow(dead_code)]
    pub fn debug_new(slugs: &Vec<String>, locale: Locale) -> Self {
        Self::new_internal(slugs, locale, true)
    }
    fn new_internal(slugs: &Vec<String>, locale: Locale, do_not_remove: bool) -> Self {
        // create wiki history file for each slug in the vector, in the configured root directory for the locale
        let mut folder_path = PathBuf::new();
        folder_path.push(root_for_locale(locale).unwrap());
        folder_path.push(locale.as_folder_str());
        fs::create_dir_all(&folder_path).unwrap();
        folder_path.push("_wikihistory.json");

        let mut entries: BTreeMap<String, Value> = BTreeMap::new();
        for slug in slugs {
            let value: BTreeMap<String, Value> = BTreeMap::from([
                (
                    "modified".to_string(),
                    Value::String(random_date_rfc3339_string()),
                ),
                ("contributors".to_string(), Value::Array(random_names())),
            ]);
            let map: serde_json::Map<String, Value> = value.into_iter().collect();
            entries.insert(slug.to_string(), Value::Object(map));
        }

        let mut json_string = serde_json::to_string_pretty(&entries).unwrap();
        json_string.push('\n');
        fs::write(&folder_path, json_string).unwrap();

        WikihistoryFixtures {
            path: folder_path,
            do_not_remove,
        }
    }
}

impl Drop for WikihistoryFixtures {
    fn drop(&mut self) {
        if self.do_not_remove {
            tracing::info!(
                "Leaving wikihistory fixture {} in place for debugging",
                self.path.display()
            );
            return;
        }

        fs::remove_file(&self.path).unwrap();
    }
}

fn random_names() -> Vec<Value> {
    let num_entries = rand::random::<u8>() % 10 + 1;
    let names: Vec<Value> = (0..num_entries)
        .map(|_| Value::String(Username().fake()))
        .collect();
    names
}

fn random_date_rfc3339_string() -> String {
    DateTimeBetween(
        DateTime::parse_from_rfc3339("2015-01-01T00:00:00Z")
            .unwrap()
            .to_utc(),
        DateTime::parse_from_rfc3339("2020-12-31T23:59:59Z")
            .unwrap()
            .to_utc(),
    )
    .fake::<DateTime<chrono::Utc>>()
    .to_rfc3339_opts(SecondsFormat::Secs, true)
}
