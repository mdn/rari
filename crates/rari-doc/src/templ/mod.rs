//! # Module Overview
//!
//! This module serves as the entry point for various submodules that handle different aspects
//! of the parsing and rendering.

pub mod api;
pub mod api_name_index;
pub mod css_feature_index;
pub mod index;
pub mod js_ref_index;
pub mod parser;
pub mod render;
pub mod templs;

use std::sync::OnceLock;

use crate::error::DocError;

fn get_or_build_index<T>(
    index: &OnceLock<T>,
    build: impl FnOnce() -> Result<T, DocError>,
) -> Result<&T, DocError> {
    if let Some(index) = index.get() {
        return Ok(index);
    }
    let built = build().map_err(|error| DocError::ReferenceIndexError(Box::new(error)))?;
    Ok(index.get_or_init(|| built))
}
