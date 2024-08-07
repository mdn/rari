use std::path::Path;

use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use rari_types::globals::{content_root, content_translated_root};
use tracing::error;

use crate::docs::page::{Page, PageReader};
use crate::error::DocError;

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

pub fn read_docs_parallel<T: PageReader>(
    paths: &[impl AsRef<Path>],
    glob: Option<&str>,
) -> Result<Vec<Page>, DocError> {
    let (tx, rx) = crossbeam_channel::bounded::<Result<Page, DocError>>(100);
    let stdout_thread = std::thread::spawn(move || rx.into_iter().collect());
    walk_builder(paths, glob)?.build_parallel().run(|| {
        let tx = tx.clone();
        Box::new(move |result| {
            if let Ok(f) = result {
                if f.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    let p = f.into_path();
                    match T::read(p) {
                        Ok(doc) => {
                            tx.send(Ok(doc)).unwrap();
                        }
                        Err(e) => {
                            error!("{e}");
                            //tx.send(Err(e.into())).unwrap();
                        }
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });

    drop(tx);
    stdout_thread.join().unwrap()
}
