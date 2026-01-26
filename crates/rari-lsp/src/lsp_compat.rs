//! Compatibility layer between `ls_types` and `lsp_types` crates.
//!
//! ## Why this exists
//!
//! `tower-lsp-server` v0.23 switched from the `lsp_types` crate to `ls_types`
//! (their fork). However, `lsp-textdocument` still depends on `lsp_types`.
//! These helper functions convert between the two type systems.
//!
//! ## When this can be removed
//!
//! This module can be removed when either:
//! - `lsp-textdocument` is updated to use `ls_types`, or
//! - An alternative text document manager compatible with `ls_types` is used
//!
//! See: <https://github.com/tower-lsp-community/tower-lsp-server/releases/tag/v0.23.0>

use tower_lsp_server::ls_types::{Position, Range, TextDocumentContentChangeEvent};

pub fn to_lsp_types_position(pos: Position) -> lsp_types::Position {
    lsp_types::Position {
        line: pos.line,
        character: pos.character,
    }
}

pub fn to_lsp_types_range(range: Range) -> lsp_types::Range {
    lsp_types::Range {
        start: to_lsp_types_position(range.start),
        end: to_lsp_types_position(range.end),
    }
}

pub fn to_lsp_types_content_change(
    change: &TextDocumentContentChangeEvent,
) -> lsp_types::TextDocumentContentChangeEvent {
    lsp_types::TextDocumentContentChangeEvent {
        range: change.range.map(to_lsp_types_range),
        range_length: change.range_length,
        text: change.text.clone(),
    }
}
