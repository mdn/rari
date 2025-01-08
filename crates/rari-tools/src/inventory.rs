use std::path::Path;

use rari_doc::error::DocError;
use rari_doc::pages::page::PageReader;
use rari_doc::pages::types::doc::FrontMatter;
use rari_doc::reader::read_docs_parallel;
use rari_doc::utils::{root_for_locale, split_fm};
use rari_types::globals::content_root;
use rari_utils::io::read_to_string;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct InventoryEntry {
    pub path: String,
    pub frontmatter: FrontMatter,
}

impl PageReader<InventoryEntry> for InventoryEntry {
    fn read(
        path: impl Into<std::path::PathBuf>,
        locale: Option<rari_types::locale::Locale>,
    ) -> Result<InventoryEntry, DocError> {
        let full_path = path.into();
        let raw = read_to_string(&full_path)?;
        let (fm, _) = split_fm(&raw);
        let fm = fm.ok_or(DocError::NoFrontmatter)?;
        let frontmatter: FrontMatter = serde_yaml_ng::from_str(fm)?;
        let path = Path::new("/")
            .join(
                full_path.strip_prefix(
                    root_for_locale(locale.unwrap_or_default())?
                        .parent()
                        .unwrap_or(Path::new(".")),
                )?,
            )
            .to_string_lossy()
            .to_string();
        Ok(InventoryEntry { path, frontmatter })
    }
}

pub fn gather_inventory() -> Result<(), DocError> {
    let inventory = read_docs_parallel::<InventoryEntry, InventoryEntry>(&[content_root()], None)?;
    let mut out = std::io::stdout();
    serde_json::to_writer_pretty(&mut out, &inventory)?;
    Ok(())
}

// These tests use file system fixtures to simulate content and translated content.
// The file system is a shared resource, so we force tests to be run serially,
// to avoid concurrent fixture management issues.
// Using `file_serial` as a synchronization lock, we should be able to run all tests
// using the same `key` (here: file_fixtures) to be serialized across modules.
#[cfg(test)]
use serial_test::file_serial;
#[cfg(test)]
#[file_serial(file_fixtures)]
mod test {
    use rari_types::locale::Locale;

    use crate::tests::fixtures::docs::DocFixtures;

    #[test]
    fn test_do_move_dry_run() {
        let slugs = vec![
            "Web/API/ExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleOne".to_string(),
            "Web/API/ExampleOne/SubExampleTwo".to_string(),
        ];
        let _docs = DocFixtures::new(&slugs, Locale::EnUs);
    }
}
