use std::path::Path;

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use rari_types::globals::{content_root, content_translated_root};

pub fn walk_builder(
    paths: &[impl AsRef<Path>],
    glob: Option<&str>,
) -> Result<WalkBuilder, ignore::Error> {
    let mut types = TypesBuilder::new();
    types.add_def(&format!("markdown:{}", glob.unwrap_or("index.md")))?;
    types.select("markdown");
    let mut paths_iter = paths.iter();
    let mut builder = if let Some(path) = paths_iter.next() {
        let mut builder = ignore::WalkBuilder::new(path);
        for path in paths_iter {
            builder.add(path);
        }
        builder
    } else {
        let mut builder = ignore::WalkBuilder::new(content_root());
        if let Some(root) = content_translated_root() {
            builder.add(root);
        }
        builder
    };
    builder.types(types.build()?);
    Ok(builder)
}
