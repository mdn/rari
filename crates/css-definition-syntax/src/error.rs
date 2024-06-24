use thiserror::Error;

use crate::parser::Node;

#[derive(Debug, Clone, Error)]
pub enum SyntaxDefinitionError {
    #[error("Expected Range node")]
    ExpectedRangeNode,
    #[error("Unknown node type {0:?}")]
    UnknownNodeType(Node),
    #[error("Parse error: Expected {0}")]
    ParseErrorExpected(char),
    #[error("Parse error: Expected function")]
    ParseErrorExpectedFunction,
    #[error("Parse error: Expected keyword")]
    ParseErrorExpectedKeyword,
    #[error("Parse error: Unexpected input")]
    ParseErrorUnexpectedInput,
}
