use std::path::Path;

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use rari_types::globals::{content_root, content_translated_root, settings};

/// Creates a `WalkBuilder` for walking through the specified paths globbing "index.md" files. The glob can be overridden.
///
/// This function initializes a `WalkBuilder` for traversing the file system starting from the given paths.
/// It supports optional glob patterns to filter the files being walked, but by default, it looks for "index.md" files.
/// For running in the test environment, the function also configures the `WalkBuilder` to respect or ignore `.gitignore`
/// files based on the settings `reader_ignores_gitignore`. Testing usually does not follow `.gitignore` files' rules.
///
/// # Arguments
///
/// * `paths` - A slice of paths to start the walk from. Each path should implement `AsRef<Path>`.
/// * `glob` - An optional string slice that holds the glob pattern to filter the files. If `None`, defaults to "index.md".
///
/// # Returns
///
/// * `Result<WalkBuilder, ignore::Error>` - Returns a configured `WalkBuilder` if successful,
///   or an `ignore::Error` if an error occurs while building the file types.
///
/// # Errors
///
/// This function will return an error if:
/// - An error occurs while adding the glob pattern to the `TypesBuilder`.
/// - An error occurs while building the file types.
pub(crate) fn walk_builder(
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
    builder.git_ignore(!settings().reader_ignores_gitignore);
    builder.types(types.build()?);
    Ok(builder)
}
