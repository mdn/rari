//! # Rari Documentation System
//!
//! The `rari_doc` crate is the central crate of the `rari` build system. It provides a robust build pipeline
//! and various utilities to handle different aspects of the documentation pipline, including reading,
//! parsing, and rendering pages.
//!
//! ## Modules
//!
//! - `baseline`: Handles baseline configurations and settings.
//! - `build`: Manages the build process for the documentation.
//! - `cached_readers`: Provides cached readers for efficient file access.
//! - `error`: Defines error types used throughout the crate.
//! - `helpers`: Contains helper functions and utilities.
//! - `html`: Manages HTML rendering and processing.
//! - `pages`: Handles the creation and management of documentation pages.
//! - `percent`: Utilities for percent encodings.
//! - `reader`: Defines traits and implementations for reading pages.
//! - `redirects`: Manages URL redirects within the documentation.
//! - `resolve`: Handles path and URL resolution.
//! - `search_index`: Manages the search index for the documentation.
//! - `sidebars`: Handles sidebar generation and management.
//! - `specs`: Manages Web-Spec and Browser Compatibility (BCD) data.
//! - `templ`: Handles templating, macros and rendering of pages.
//! - `translations`: Tools for efficiently looking up translated documents.
//! - `utils`: Contains various utility functions.
//! - `walker`: Provides functionality to walk through the documentation file tree.
//!
//! ## Introduction to Rari Pages and Build Pipeline
//!
//! Rari pages are the core components of the documentation system. Each page can be read,
//! parsed, and rendered using the various modules provided
//! by the `rari_doc` crate. The build pipeline is designed to efficiently process these pages,
//! handling tasks such as reading from source files, applying templates, managing translations,
//! and generating the final output.
pub mod baseline;
pub mod build;
pub mod cached_readers;
pub mod error;
pub mod helpers;
pub mod html;
pub mod pages;
pub mod percent;
pub mod reader;
pub mod redirects;
pub mod resolve;
pub mod search_index;
pub mod sidebars;
pub mod specs;
pub mod templ;
pub mod translations;
pub mod utils;
pub mod walker;
