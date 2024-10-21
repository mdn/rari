use std::path::Path;

use rari_types::globals::settings;
use tracing::error;

use crate::error::DocError;
use crate::pages::page::{Page, PageReader};
use crate::walker::walk_builder;

pub fn read_docs_parallel<T: PageReader>(
    paths: &[impl AsRef<Path>],
    glob: Option<&str>,
) -> Result<Vec<Page>, DocError> {
    let (tx, rx) = crossbeam_channel::bounded::<Result<Page, DocError>>(100);
    let stdout_thread = std::thread::spawn(move || rx.into_iter().collect());
    // For testing, we do not pay attention to the .gitignore files (walk_builder's
    // default is to obey them). The test configuration has `reader_ignores_gitignore = true`.
    let ignore_gitignore = !settings().reader_ignores_gitignore;
    walk_builder(paths, glob)?
        .git_ignore(ignore_gitignore)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            Box::new(move |result| {
                if let Ok(f) = result {
                    if f.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        let p = f.into_path();
                        match T::read(p, None) {
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
