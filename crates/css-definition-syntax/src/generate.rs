use crate::error::SyntaxDefinitionError;
use crate::parser::{Group, Node, Range};

fn generate_multiplier_simple(min: u32, max: u32, comma: bool) -> Option<&'static str> {
    match (min, max) {
        (0, 0) if comma => Some("#?"),
        (0, 0) => Some("*"),
        (0, 1) => Some("?"),
        (1, 0) if comma => Some("#"),
        (1, 0) => Some("+"),
        (1, 1) => Some(""),
        _ => None,
    }
}
fn generate_multiplier(min: u32, max: u32, comma: bool) -> String {
    if let Some(result) = generate_multiplier_simple(min, max, comma) {
        return result.to_string();
    }

    let number_sign = if comma { "#" } else { "" };
    match (min, max) {
        (min, max) if min == max => format!("{}{{{}}}", number_sign, min),
        (min, 0) => format!("{}{{{},}}", number_sign, min),
        (min, max) => format!("{}{{{},{}}}", number_sign, min, max),
    }
}

fn generate_type_opts(node: &Node) -> Result<String, SyntaxDefinitionError> {
    if let Node::Range(Range {
        min,
        max,
        min_unit,
        max_unit,
    }) = node
    {
        let min_unit = min_unit
            .as_ref()
            .and_then(|unit| {
                if min.is_finite() {
                    Some(unit.as_str())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let max_unit = max_unit
            .as_ref()
            .and_then(|unit| {
                if max.is_finite() {
                    Some(unit.as_str())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        Ok(format!(" [{min}{min_unit},{max}{max_unit}]"))
    } else {
        Err(SyntaxDefinitionError::ExpectedRangeNode)
    }
}

fn internal_generate<'a>(
    node: &'a Node,
    decorate: DecorateFn<'a>,
    force_braces: bool,
    compact: bool,
) -> Result<String, SyntaxDefinitionError> {
    let out = match node {
        Node::Multiplier(multiplier) => {
            let terms = internal_generate(&multiplier.term, decorate, force_braces, compact)?;
            let multiplier = generate_multiplier(multiplier.min, multiplier.max, multiplier.comma);
            let decorated = decorate(multiplier, node);
            format!("{}{}", terms, decorated)
        }
        Node::Token(token) => token.value.to_string(),
        Node::Property(property) => format!("<'{}'>", property.name),
        Node::Type(typ) => {
            let opts = if let Some(opts) = &typ.opts {
                Some(decorate(generate_type_opts(opts)?, opts))
            } else {
                None
            };
            format!("<{}{}>", typ.name, opts.as_deref().unwrap_or_default())
        }
        Node::Function(function) => format!("{}(", function.name),
        Node::Keyword(keyword) => keyword.name.clone(),
        Node::Comma => ",".to_string(),
        Node::String(s) => s.value.clone(),
        Node::AtKeyword(at_keyword) => format!("@{}", at_keyword.name),
        Node::Group(group) => {
            format!(
                "{}{}",
                generate_sequence(group, decorate, force_braces, compact)?,
                if group.disallow_empty { "!" } else { "" }
            )
        }
        n => Err(SyntaxDefinitionError::UnknownNodeType(n.clone()))?,
    };

    Ok(decorate(out, node))
}

fn generate_sequence<'a>(
    group: &'a Group,
    decorate: DecorateFn<'a>,
    force_braces: bool,
    compact: bool,
) -> Result<String, SyntaxDefinitionError> {
    let combinator = if compact {
        group.combinator.as_str_compact()
    } else {
        group.combinator.as_str()
    };

    let result = group
        .terms
        .iter()
        .map(|node| internal_generate(node, decorate, force_braces, compact))
        .collect::<Result<Vec<_>, _>>()?
        .join(combinator);
    if group.explicit || force_braces {
        let start = if compact || result.starts_with(',') {
            "["
        } else {
            "[ "
        };
        let end = if compact { "]" } else { " ]" };
        Ok(format!("{}{}{}", start, result, end))
    } else {
        Ok(result)
    }
}

fn noop(s: String, _: &Node) -> String {
    s
}

pub type DecorateFn<'a> = &'a dyn Fn(String, &'a Node) -> String;

pub struct GenerateOptions<'a> {
    pub compact: bool,
    pub force_braces: bool,
    pub decorate: DecorateFn<'a>,
}

impl<'a> Default for GenerateOptions<'a> {
    fn default() -> Self {
        Self {
            compact: Default::default(),
            force_braces: Default::default(),
            decorate: &noop,
        }
    }
}

pub fn generate<'a>(
    node: &'a Node,
    options: GenerateOptions<'a>,
) -> Result<String, SyntaxDefinitionError> {
    internal_generate(
        node,
        options.decorate,
        options.force_braces,
        options.compact,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_generate() -> Result<(), SyntaxDefinitionError> {
        let input = "[<foo> |   <bar>{0,0}] <baz>";
        let node = parse(input)?;
        let result = generate(&node, Default::default()).unwrap();
        assert_eq!(result, "[ <foo> | <bar>* ] <baz>");

        let input = "<foo>+#{1,2}";
        let node = parse(input)?;
        let result = generate(
            &node,
            GenerateOptions {
                decorate: &|s, _| format!("!!{}¡¡", s),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result, "!!!!!!!!<foo>¡¡!!+¡¡¡¡!!#{1,2}¡¡¡¡¡¡");

        let input = "<foo [0,∞]>";
        let node = parse(input)?;
        let result = generate(&node, Default::default()).unwrap();
        assert_eq!(result, "<foo [0,∞]>");

        let input = "<calc-product> [ [ '+' | '-' ] <calc-product> ]*";
        let node = parse(input)?;
        let result = generate(&node, Default::default()).unwrap();
        assert_eq!(result, "<calc-product> [ [ '+' | '-' ] <calc-product> ]*");
        Ok(())
    }
}
