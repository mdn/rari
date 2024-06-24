use std::fmt;

use css_definition_syntax::error::SyntaxDefinitionError;
use css_definition_syntax::parser::Node;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum SyntaxError {
    #[error(transparent)]
    SyntaxDefinitionError(#[from] SyntaxDefinitionError),
    #[error("Expected group node, got: {}", .0.str_name())]
    ExpectedGroupNode(Node),
    #[error("IoError")]
    IoError,
    #[error("fmtError")]
    FmtError(#[from] fmt::Error),
    #[error("Error: could not find syntax for this item")]
    NoSyntaxFound,
}
