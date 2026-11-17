//! # Module Overview
//!
//! This module serves as the entry point for various submodules that handle different aspects
//! of the parsing and rendering.

pub mod api;
pub mod api_name_index;
pub mod css_feature_index;
pub mod js_ref_index;
pub mod parser;
pub mod render;
pub mod templs;

use std::sync::{Mutex, OnceLock, PoisonError};

use crate::error::DocError;

/// A lazily built reference index.
struct LazyIndex<T> {
    index: OnceLock<T>,
    building: Mutex<()>,
}

impl<T> LazyIndex<T> {
    const fn new() -> Self {
        Self {
            index: OnceLock::new(),
            building: Mutex::new(()),
        }
    }

    fn get_or_build(&self, build: impl FnOnce() -> Result<T, DocError>) -> Result<&T, DocError> {
        if let Some(index) = self.index.get() {
            return Ok(index);
        }
        // Concurrent first users wait for one build instead of each running their own.
        let _building = self.building.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(index) = self.index.get() {
            return Ok(index);
        }
        let built = build().map_err(|error| DocError::ReferenceIndexError(Box::new(error)))?;
        Ok(self.index.get_or_init(|| built))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Barrier;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn concurrent_first_users_build_once() {
        const THREADS: usize = 8;
        let index = LazyIndex::<usize>::new();
        let builds = AtomicUsize::new(0);
        let barrier = Barrier::new(THREADS);

        thread::scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    barrier.wait();
                    let value = index
                        .get_or_build(|| {
                            thread::sleep(Duration::from_millis(20));
                            Ok(builds.fetch_add(1, Ordering::SeqCst))
                        })
                        .unwrap();
                    assert_eq!(*value, 0);
                });
            }
        });

        assert_eq!(builds.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failed_build_is_retried() {
        let index = LazyIndex::<usize>::new();
        assert!(index.get_or_build(|| Err(DocError::NotADoc)).is_err());
        assert_eq!(*index.get_or_build(|| Ok(1)).unwrap(), 1);
    }
}
