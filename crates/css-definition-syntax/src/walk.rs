use crate::error::SyntaxDefinitionError;
use crate::parser::Node;

pub struct WalkOptions<T> {
    pub enter: fn(&Node, &mut T) -> Result<(), SyntaxDefinitionError>,
    pub leave: fn(&Node, &mut T) -> Result<(), SyntaxDefinitionError>,
}

fn noop<T>(_: &Node, _: &mut T) -> Result<(), SyntaxDefinitionError> {
    Ok(())
}

impl<T> Default for WalkOptions<T> {
    fn default() -> Self {
        Self {
            enter: noop,
            leave: noop,
        }
    }
}

pub fn walk<T>(
    node: &Node,
    options: &WalkOptions<T>,
    context: &mut T,
) -> Result<(), SyntaxDefinitionError> {
    (options.enter)(node, context)?;
    match node {
        Node::Group(group) => {
            for term in &group.terms {
                walk(term, options, context)?;
            }
        }
        Node::Multiplier(multiplier) => {
            walk(&multiplier.term, options, context)?;
        }
        Node::BooleanExpr(boolean_exp) => {
            walk(&boolean_exp.term, options, context)?;
        }
        Node::Token(_)
        | Node::Property(_)
        | Node::Type(_)
        | Node::Function(_)
        | Node::Keyword(_)
        | Node::Comma
        | Node::String(_)
        | Node::AtKeyword(_) => {}
        _ => Err(SyntaxDefinitionError::UnknownNodeType(node.clone()))?,
    }
    (options.leave)(node, context)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_walk() -> Result<(), SyntaxDefinitionError> {
        let syntax = parse("<foo> | <bar>{0,0} <baz>")?;

        walk(
            &syntax,
            &WalkOptions {
                enter: |_, _| Ok(()),
                leave: |_, _| Ok(()),
            },
            &mut (),
        )?;
        Ok(())
    }
}
