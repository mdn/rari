use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::Display;
use std::iter::empty;

use crate::error::SyntaxDefinitionError;
use crate::tokenizer::Tokenizer;

const TAB: char = '\t';
const N: char = '\n';
const F: char = '\u{c}';
const R: char = '\r';
const SPACE: char = ' ';
const EXCLAMATION_MARK: char = '!';
const NUMBER_SIGN: char = '#';
const AMPERSAND: char = '&';
const APOSTROPHE: char = '\'';
const LEFT_PARENTHESIS: char = '(';
const RIGHT_PARENTHESIS: char = ')';
const ASTERISK: char = '*';
const PLUS_SIGN: char = '+';
const COMMA: char = ',';
const HYPER_MINUS: char = '-';
const LESS_THAN_SIGN: char = '<';
const GREATER_THAN_SIGN: char = '>';
const QUESTION_MARK: char = '?';
const COMMERCIAL_AT: char = '@';
const LEFT_SQUARE_BRACKET: char = '[';
const RIGHT_SQUARE_BRACKET: char = ']';
const LEFT_CURLY_BRACKET: char = '{';
const VERTICAL_LINE: char = '|';
const RIGHT_CURLY_BRACKET: char = '}';
const INFINITY: char = '∞';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CombinatorType {
    Space,
    DoubleAmpersand,
    DoubleVerticalLine,
    VerticalLine,
}

impl CombinatorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CombinatorType::Space => " ",
            CombinatorType::DoubleAmpersand => " && ",
            CombinatorType::DoubleVerticalLine => " || ",
            CombinatorType::VerticalLine => " | ",
        }
    }

    pub fn as_str_compact(&self) -> &'static str {
        match self {
            CombinatorType::Space => " ",
            CombinatorType::DoubleAmpersand => "&&",
            CombinatorType::DoubleVerticalLine => "||",
            CombinatorType::VerticalLine => "|",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaybeMultiplier {
    pub comma: bool,
    pub min: u32,
    pub max: u32,
    pub term: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Multiplier {
    pub comma: bool,
    pub min: u32,
    pub max: u32,
    pub term: Box<Node>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub value: char,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Property {
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Range {
    pub min: IntI<i32>,
    pub max: IntI<i32>,
    pub min_unit: Option<String>,
    pub max_unit: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub name: String,
    pub opts: Option<Box<Node>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Function {
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keyword {
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Combinator {
    pub value: CombinatorType,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringNode {
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spaces {
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtKeyword {
    pub name: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    pub terms: Vec<Node>,
    pub combinator: CombinatorType,
    pub disallow_empty: bool,
    pub explicit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Multiplier(Multiplier),
    Token(Token),
    Property(Property),
    Range(Range),
    Type(Type),
    Function(Function),
    Keyword(Keyword),
    Combinator(Combinator),
    Comma,
    String(StringNode),
    Spaces(Spaces),
    AtKeyword(AtKeyword),
    Group(Group),
}

impl Node {
    pub fn str_name(&self) -> &str {
        match self {
            Node::Multiplier(_) => "Multiplier",
            Node::Token(_) => "Token",
            Node::Property(_) => "Property",
            Node::Range(_) => "Range",
            Node::Type(_) => "Type",
            Node::Function(_) => "Function",
            Node::Keyword(_) => "Keyword",
            Node::Combinator(_) => "Combinator",
            Node::Comma => "Comma",
            Node::String(_) => "String",
            Node::Spaces(_) => "Spaces",
            Node::AtKeyword(_) => "AtKeyword",
            Node::Group(_) => "Group",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IntI<T> {
    Finite(T),
    Infinity,
    NegativeInfinity,
}
impl<T> IntI<T> {
    pub fn is_finite(&self) -> bool {
        matches!(self, IntI::Finite(_))
    }
}

impl<T> Display for IntI<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IntI::*;
        match self {
            Finite(x) => write!(f, "{}", x),
            Infinity => write!(f, "∞"),
            NegativeInfinity => write!(f, "-∞"),
        }
    }
}

impl<T> PartialOrd for IntI<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use IntI::*;
        match (self, other) {
            (Infinity, Infinity) | (NegativeInfinity, NegativeInfinity) => Some(Ordering::Equal),
            (Infinity, _) | (_, NegativeInfinity) => Some(Ordering::Greater),
            (NegativeInfinity, _) | (_, Infinity) => Some(Ordering::Less),
            (Finite(xf), Finite(yf)) => xf.partial_cmp(yf),
        }
    }
}

const fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}

fn scan_spaces(tokenizer: &mut Tokenizer) -> String {
    tokenizer.substring_to_pos(tokenizer.find_ws_end(tokenizer.pos))
}

fn scan_word(
    tokenizer: &mut Tokenizer,
    can_be_last: bool,
) -> Result<String, SyntaxDefinitionError> {
    let end = tokenizer
        .str
        .iter()
        .skip(tokenizer.pos)
        .position(|c| !is_name_char(*c))
        .map(|pos| pos + tokenizer.pos)
        .or(if can_be_last {
            Some(tokenizer.str.len())
        } else {
            None
        })
        .ok_or(SyntaxDefinitionError::ParseErrorExpectedKeyword)?;

    Ok(tokenizer.substring_to_pos(end))
}

fn scan_number(tokenizer: &mut Tokenizer) -> String {
    let end = tokenizer
        .str
        .iter()
        .skip(tokenizer.pos)
        .position(|c| !c.is_ascii_digit())
        .map(|pos| pos + tokenizer.pos)
        .unwrap_or_else(|| {
            tokenizer.pos = tokenizer.str.len();
            tokenizer.error("Expect a number");
            tokenizer.str.len()
        });

    tokenizer.substring_to_pos(end)
}

fn scan_string(tokenizer: &mut Tokenizer) -> String {
    let end = tokenizer
        .str
        .iter()
        .skip(tokenizer.pos + 1)
        .position(|c| *c == '\'')
        .map(|pos| pos + tokenizer.pos)
        .unwrap_or_else(|| {
            tokenizer.pos = tokenizer.str.len();
            tokenizer.error("Expect an apostrophe");
            0
        });

    tokenizer.substring_to_pos(end + 2)
}

pub struct MultiplierRange {
    pub min: u32,
    pub max: u32,
}

// TODO: This should ignore whitespace and comments
// See https://www.w3.org/TR/css-values-4/#component-multipliers
fn read_multiplier_range(
    tokenizer: &mut Tokenizer,
) -> Result<MultiplierRange, SyntaxDefinitionError> {
    tokenizer.eat(LEFT_CURLY_BRACKET)?;
    let min = scan_number(tokenizer).parse::<u32>().unwrap();

    let max = if tokenizer.char_code() == COMMA {
        tokenizer.pos += 1;
        if tokenizer.char_code() != RIGHT_CURLY_BRACKET {
            scan_number(tokenizer).parse::<u32>().unwrap()
        } else {
            0
        }
    } else {
        min
    };

    tokenizer.eat(RIGHT_CURLY_BRACKET)?;

    Ok(MultiplierRange { min, max })
}

fn read_multiplier(
    tokenizer: &mut Tokenizer,
) -> Result<Option<MaybeMultiplier>, SyntaxDefinitionError> {
    let mut comma = false;
    let range = match tokenizer.char_code() {
        '*' => {
            tokenizer.pos += 1;
            MultiplierRange { min: 0, max: 0 }
        }
        '+' => {
            tokenizer.pos += 1;
            MultiplierRange { min: 1, max: 0 }
        }
        '?' => {
            tokenizer.pos += 1;
            MultiplierRange { min: 0, max: 1 }
        }
        '#' => {
            tokenizer.pos += 1;
            comma = true;
            if tokenizer.char_code() == LEFT_CURLY_BRACKET {
                read_multiplier_range(tokenizer)?
            } else if tokenizer.char_code() == '?' {
                tokenizer.pos += 1;
                MultiplierRange { min: 0, max: 0 }
            } else {
                MultiplierRange { min: 1, max: 0 }
            }
        }
        '{' => read_multiplier_range(tokenizer)?,

        _ => return Ok(None),
    };

    Ok(Some(MaybeMultiplier {
        comma,
        min: range.min,
        max: range.max,
        term: None,
    }))
}

fn maybe_multiplied(tokenizer: &mut Tokenizer, node: Node) -> Result<Node, SyntaxDefinitionError> {
    let multiplier = read_multiplier(tokenizer)?;
    if let Some(MaybeMultiplier {
        comma,
        min,
        max,
        term: _,
    }) = multiplier
    {
        // https://www.w3.org/TR/css-values-4/#component-multipliers
        // > The + and # multipliers may be stacked as +#;
        // Represent "+#" as nested multipliers:
        // { ...<multiplier #>,
        //   term: {
        //     ...<multiplier +>,
        //     term: node
        //   }
        // }
        if tokenizer.char_code() == NUMBER_SIGN
            && tokenizer.char_code_at(tokenizer.pos - 1) == PLUS_SIGN
        {
            return maybe_multiplied(
                tokenizer,
                Node::Multiplier(Multiplier {
                    comma,
                    min,
                    max,
                    term: Box::new(node),
                }),
            );
        }
        return Ok(Node::Multiplier(Multiplier {
            comma,
            min,
            max,
            term: Box::new(node),
        }));
    }
    Ok(node)
}

fn maybe_token(tokenizer: &mut Tokenizer) -> Option<Node> {
    let ch = tokenizer.peek();
    if ch == '\0' {
        return None;
    }
    Some(Node::Token(Token { value: ch }))
}

fn read_property(tokenizer: &mut Tokenizer) -> Result<Node, SyntaxDefinitionError> {
    tokenizer.eat(LESS_THAN_SIGN)?;
    tokenizer.eat(APOSTROPHE)?;

    let name = scan_word(tokenizer, false)?;

    tokenizer.eat(APOSTROPHE)?;
    tokenizer.eat(GREATER_THAN_SIGN)?;

    maybe_multiplied(tokenizer, Node::Property(Property { name }))
}

// https://drafts.csswg.org/css-values-3/#numeric-ranges
// 4.1. Range Restrictions and Range Definition Notation
//
// Range restrictions can be annotated in the numeric type notation using CSS bracketed
// range notation—[min,max]—within the angle brackets, after the identifying keyword,
// indicating a closed range between (and including) min and max.
// For example, <integer [0, 10]> indicates an integer between 0 and 10, inclusive.
fn read_type_range(tokenizer: &mut Tokenizer) -> Result<Node, SyntaxDefinitionError> {
    tokenizer.eat(LEFT_SQUARE_BRACKET)?;

    let sign = if tokenizer.char_code() == HYPER_MINUS {
        tokenizer.peek();
        -1
    } else {
        1
    };

    let (min, min_unit) = if sign == -1 && tokenizer.char_code() == INFINITY {
        tokenizer.peek();
        (IntI::NegativeInfinity, None)
    } else {
        let min = scan_number(tokenizer)
            .parse::<i32>()
            .map(|x| IntI::Finite(x * sign))
            .unwrap_or(IntI::NegativeInfinity);

        if is_name_char(tokenizer.char_code()) {
            (min, Some(scan_word(tokenizer, false)?))
        } else {
            (min, None)
        }
    };

    scan_spaces(tokenizer);
    tokenizer.eat(COMMA)?;
    scan_spaces(tokenizer);

    let (max, max_unit) = if tokenizer.char_code() == INFINITY {
        tokenizer.peek();
        (IntI::Infinity, None)
    } else {
        let sign = if tokenizer.char_code() == HYPER_MINUS {
            tokenizer.peek();
            -1
        } else {
            1
        };

        let max = scan_number(tokenizer)
            .parse::<i32>()
            .map(|x| IntI::Finite(x * sign))
            .unwrap_or(IntI::Infinity);

        if is_name_char(tokenizer.char_code()) {
            (max, Some(scan_word(tokenizer, false)?))
        } else {
            (max, None)
        }
    };

    if min_unit.is_some() && max_unit.is_some() && min_unit != max_unit {
        tokenizer.error("Mismatched units in range");
    }

    tokenizer.eat(RIGHT_SQUARE_BRACKET)?;

    Ok(Node::Range(Range {
        min,
        max,
        min_unit,
        max_unit,
    }))
}

fn read_type(tokenizer: &mut Tokenizer) -> Result<Node, SyntaxDefinitionError> {
    tokenizer.eat(LESS_THAN_SIGN)?;
    let mut name = scan_word(tokenizer, false)?;

    if tokenizer.char_code() == LEFT_PARENTHESIS && tokenizer.next_char_code() == RIGHT_PARENTHESIS
    {
        tokenizer.pos += 2;
        name.push_str("()")
    }

    let opts =
        if tokenizer.char_code_at(tokenizer.find_ws_end(tokenizer.pos)) == LEFT_SQUARE_BRACKET {
            scan_spaces(tokenizer);
            Some(Box::new(read_type_range(tokenizer)?))
        } else {
            None
        };
    tokenizer.eat(GREATER_THAN_SIGN)?;

    maybe_multiplied(tokenizer, Node::Type(Type { name, opts }))
}

fn read_keyword_or_function(tokenizer: &mut Tokenizer) -> Result<Node, SyntaxDefinitionError> {
    let name = scan_word(tokenizer, true)?;

    if tokenizer.char_code() == LEFT_PARENTHESIS {
        tokenizer.pos += 1;
        if tokenizer.pos >= tokenizer.str.len() {
            return Err(SyntaxDefinitionError::ParseErrorExpectedFunction);
        }

        return Ok(Node::Function(Function { name }));
    }

    maybe_multiplied(tokenizer, Node::Keyword(Keyword { name }))
}

fn regroup_terms(
    mut terms: Vec<Node>,
    combinators: HashSet<CombinatorType>,
) -> (Vec<Node>, CombinatorType) {
    let mut combinators = combinators.into_iter().collect::<Vec<CombinatorType>>();
    combinators.sort();
    combinators.reverse();

    let combinator = combinators
        .first()
        .copied()
        .unwrap_or(CombinatorType::Space);

    while let Some(combinator) = combinators.pop() {
        let mut i = 0;
        let mut subgroup_start: Option<usize> = None;

        while i < terms.len() {
            let term = &terms[i];

            if let Node::Combinator(Combinator { value }) = term {
                if *value == combinator {
                    if subgroup_start.is_none() {
                        subgroup_start = if i > 0 { Some(i - 1) } else { None };
                    }
                    terms.remove(i);
                    continue;
                } else {
                    if let Some(subgroup_start) = subgroup_start {
                        if i - subgroup_start > 1 {
                            let group = terms.splice(subgroup_start..i, empty()).collect();
                            terms.insert(
                                subgroup_start,
                                Node::Group(Group {
                                    terms: group,
                                    combinator,
                                    disallow_empty: false,
                                    explicit: false,
                                }),
                            );
                            i = subgroup_start + 1;
                        }
                    }
                    subgroup_start = None;
                }
            }
            i += 1;
        }

        if let Some(subgroup_start) = subgroup_start {
            if !combinators.is_empty() {
                let group = terms.splice(subgroup_start..i, empty()).collect();
                terms.insert(
                    subgroup_start,
                    Node::Group(Group {
                        terms: group,
                        combinator,
                        disallow_empty: false,
                        explicit: false,
                    }),
                );
            }
        }
    }
    (terms, combinator)
}

fn read_implicit_group(tokenizer: &mut Tokenizer) -> Result<Group, SyntaxDefinitionError> {
    let mut prev_token_pos = tokenizer.pos;
    let mut combinators = HashSet::new();
    let mut terms = vec![];

    while let Some(token) = peek(tokenizer)? {
        match (&token, terms.last()) {
            (Node::Spaces(Spaces { value: _ }), _) => continue,
            (
                Node::Combinator(Combinator { value: _ }),
                Some(Node::Combinator(Combinator { value: _ })) | None,
            ) => {
                tokenizer.pos = prev_token_pos;
                tokenizer.error("Unexpected combinator");
            }
            (Node::Combinator(Combinator { value }), _) => {
                combinators.insert(*value);
            }
            (_, Some(Node::Combinator(Combinator { value: _ })) | None) => {}
            _ => {
                combinators.insert(CombinatorType::Space);
                terms.push(Node::Combinator(Combinator {
                    value: CombinatorType::Space,
                }));
            }
        }
        terms.push(token);
        prev_token_pos = tokenizer.pos;
    }

    if let Some(Node::Combinator(Combinator { value: _ })) = terms.last() {
        tokenizer.pos = prev_token_pos;
        tokenizer.error("Unexpected combinator");
    }
    let (terms, combinator) = regroup_terms(terms, combinators);
    Ok(Group {
        terms,
        combinator,
        disallow_empty: false,
        explicit: false,
    })
}

fn read_group(tokenizer: &mut Tokenizer) -> Result<Node, SyntaxDefinitionError> {
    tokenizer.eat(LEFT_SQUARE_BRACKET)?;
    let mut group = read_implicit_group(tokenizer)?;
    tokenizer.eat(RIGHT_SQUARE_BRACKET)?;

    group.explicit = true;

    if tokenizer.char_code() == EXCLAMATION_MARK {
        tokenizer.pos += 1;
        group.disallow_empty = true;
    }

    Ok(Node::Group(group))
}

fn peek(tokenizer: &mut Tokenizer) -> Result<Option<Node>, SyntaxDefinitionError> {
    let code = tokenizer.char_code();
    if is_name_char(code) {
        return Ok(Some(read_keyword_or_function(tokenizer)?));
    }

    Ok(match code {
        RIGHT_SQUARE_BRACKET => None,
        LEFT_SQUARE_BRACKET => {
            let group = read_group(tokenizer)?;
            Some(maybe_multiplied(tokenizer, group)?)
        }
        LESS_THAN_SIGN => Some(if tokenizer.next_char_code() == APOSTROPHE {
            read_property(tokenizer)?
        } else {
            read_type(tokenizer)?
        }),
        VERTICAL_LINE => {
            tokenizer.eat(VERTICAL_LINE)?;
            let value = if tokenizer.char_code() == VERTICAL_LINE {
                tokenizer.eat(VERTICAL_LINE)?;
                CombinatorType::DoubleVerticalLine
            } else {
                CombinatorType::VerticalLine
            };

            Some(Node::Combinator(Combinator { value }))
        }
        AMPERSAND => {
            tokenizer.pos += 1;
            tokenizer.eat(AMPERSAND)?;
            Some(Node::Combinator(Combinator {
                value: CombinatorType::DoubleAmpersand,
            }))
        }
        COMMA => {
            tokenizer.pos += 1;
            Some(Node::Comma)
        }
        APOSTROPHE => {
            let value = scan_string(tokenizer);
            Some(maybe_multiplied(
                tokenizer,
                Node::String(StringNode { value }),
            )?)
        }
        SPACE | TAB | N | R | F => {
            let value = scan_spaces(tokenizer);
            Some(Node::Spaces(Spaces { value }))
        }
        COMMERCIAL_AT => {
            let code = tokenizer.next_char_code();
            if is_name_char(code) {
                tokenizer.pos += 1;
                Some(Node::AtKeyword(AtKeyword {
                    name: scan_word(tokenizer, true)?,
                }))
            } else {
                maybe_token(tokenizer)
            }
        }
        ASTERISK | PLUS_SIGN | QUESTION_MARK | NUMBER_SIGN | EXCLAMATION_MARK => None,
        LEFT_CURLY_BRACKET => {
            let code = tokenizer.next_char_code();
            if !code.is_ascii_digit() {
                maybe_token(tokenizer)
            } else {
                None
            }
        }

        _ => maybe_token(tokenizer),
    })
}

pub fn parse(source: &str) -> Result<Node, SyntaxDefinitionError> {
    let mut tokenizer = Tokenizer::new(source);
    let mut result = read_implicit_group(&mut tokenizer)?;

    if tokenizer.pos != tokenizer.str.len() {
        return Err(SyntaxDefinitionError::ParseErrorUnexpectedInput);
    }

    Ok(
        if matches!(result.terms.as_slice(), &[Node::Group(_)]) && result.terms.len() == 1 {
            result.terms.pop().unwrap()
        } else {
            Node::Group(result)
        },
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scan_spaces() {
        let mut tokenizer = Tokenizer::new("  \t\nfoo");
        let result = scan_spaces(&mut tokenizer);
        assert_eq!(result, "  \t\n");
    }

    #[test]
    fn test_scan_string() {
        let mut tokenizer = Tokenizer::new("hello' 123 'foo'");
        let result = scan_string(&mut tokenizer);
        assert_eq!(result, "hello'");
    }

    #[test]
    fn test_scan_number() {
        let mut tokenizer = Tokenizer::new("'hello' 123 'foo'");
        tokenizer.pos = 8;
        let result = scan_number(&mut tokenizer);
        assert_eq!(result, "123");
    }

    #[test]
    fn test_scan_word() -> Result<(), SyntaxDefinitionError> {
        let mut tokenizer = Tokenizer::new("color 123 'foo'");
        let result = scan_word(&mut tokenizer, false)?;
        assert_eq!(result, "color");
        Ok(())
    }

    #[test]
    fn test_scan_multiplier_range() -> Result<(), SyntaxDefinitionError> {
        let mut tokenizer = Tokenizer::new("{1,2}");
        let result = read_multiplier_range(&mut tokenizer)?;
        assert_eq!(result.min, 1);
        assert_eq!(result.max, 2);

        let mut tokenizer = Tokenizer::new("{1,}");
        let result = read_multiplier_range(&mut tokenizer)?;
        assert_eq!(result.min, 1);
        assert_eq!(result.max, 0);

        let mut tokenizer = Tokenizer::new("{1}");
        let result = read_multiplier_range(&mut tokenizer)?;
        assert_eq!(result.min, 1);
        assert_eq!(result.max, 1);
        Ok(())
    }

    #[test]
    fn test_read_multiplier() -> Result<(), SyntaxDefinitionError> {
        let mut tokenizer = Tokenizer::new("#{1,4}");
        if let Some(MaybeMultiplier {
            comma,
            min,
            max,
            term: _,
        }) = read_multiplier(&mut tokenizer)?
        {
            assert_eq!(min, 1);
            assert_eq!(max, 4);
            assert!(comma);
        } else {
            panic!("Expected a multiplier");
        }
        Ok(())
    }

    #[test]
    fn test_read_range() -> Result<(), SyntaxDefinitionError> {
        let mut tokenizer = Tokenizer::new("[1,2]");
        if let Node::Range(Range {
            min,
            max,
            min_unit,
            max_unit,
        }) = read_type_range(&mut tokenizer)?
        {
            assert_eq!(min, IntI::Finite(1));
            assert_eq!(max, IntI::Finite(2));
            assert_eq!(min_unit, None);
            assert_eq!(max_unit, None);
        } else {
            panic!("Expected a range");
        }

        let mut tokenizer = Tokenizer::new("[-∞,2]");
        if let Node::Range(Range {
            min,
            max,
            min_unit,
            max_unit,
        }) = read_type_range(&mut tokenizer)?
        {
            assert_eq!(min, IntI::NegativeInfinity);
            assert_eq!(max, IntI::Finite(2));
            assert_eq!(min_unit, None);
            assert_eq!(max_unit, None);
        } else {
            panic!("Expected a range");
        }

        let mut tokenizer = Tokenizer::new("[-100deg,∞]");
        if let Node::Range(Range {
            min,
            max,
            min_unit,
            max_unit,
        }) = read_type_range(&mut tokenizer)?
        {
            assert_eq!(min, IntI::Finite(-100));
            assert_eq!(max, IntI::Infinity);
            assert_eq!(min_unit, Some("deg".to_string()));
            assert_eq!(max_unit, None);
        } else {
            panic!("Expected a range");
        }
        Ok(())
    }

    #[test]
    fn test_read_type() -> Result<(), SyntaxDefinitionError> {
        let mut tokenizer = Tokenizer::new("<integer>");
        if let Node::Type(Type { name, opts }) = read_type(&mut tokenizer)? {
            assert_eq!(name, "integer");
            assert_eq!(opts, None);
        } else {
            panic!("Expected a type");
        }

        let mut tokenizer = Tokenizer::new("<integer [0,10]>");
        if let Node::Type(Type { name, opts }) = read_type(&mut tokenizer)? {
            assert_eq!(name, "integer");
            assert_eq!(
                opts,
                Some(Box::new(Node::Range(Range {
                    min: IntI::Finite(0),
                    max: IntI::Finite(10),
                    min_unit: None,
                    max_unit: None,
                })))
            );
        } else {
            panic!("Expected a type");
        }

        Ok(())
    }

    #[test]
    fn test_combinator_order() {
        let mut combinators = vec![
            CombinatorType::DoubleVerticalLine,
            CombinatorType::Space,
            CombinatorType::VerticalLine,
            CombinatorType::DoubleAmpersand,
        ];

        combinators.sort();

        assert_eq!(
            combinators,
            vec![
                CombinatorType::Space,
                CombinatorType::DoubleAmpersand,
                CombinatorType::DoubleVerticalLine,
                CombinatorType::VerticalLine
            ]
        );
    }

    #[test]
    fn test_parse_simple() -> Result<(), SyntaxDefinitionError> {
        let result = parse("<color> | <integer> | <percentage>")?;
        assert_eq!(
            result,
            Node::Group(Group {
                terms: vec![
                    Node::Type(Type {
                        name: "color".to_string(),
                        opts: None
                    }),
                    Node::Type(Type {
                        name: "integer".to_string(),
                        opts: None
                    }),
                    Node::Type(Type {
                        name: "percentage".to_string(),
                        opts: None
                    })
                ],
                combinator: CombinatorType::VerticalLine,
                disallow_empty: false,
                explicit: false
            })
        );
        Ok(())
    }

    #[test]
    fn test_parse_complex() -> Result<(), SyntaxDefinitionError> {
        let syntax = "a b | c() && [ <d>? || <'e'> || ( f{2,4} ) ]*";
        let result = parse(syntax)?;
        assert_eq!(
            result,
            Node::Group(Group {
                terms: vec![
                    Node::Group(Group {
                        terms: vec![
                            Node::Keyword(Keyword {
                                name: "a".to_string()
                            }),
                            Node::Keyword(Keyword {
                                name: "b".to_string()
                            })
                        ],
                        combinator: CombinatorType::Space,
                        disallow_empty: false,
                        explicit: false,
                    }),
                    Node::Group(Group {
                        terms: vec![
                            Node::Group(Group {
                                terms: vec![
                                    Node::Function(Function {
                                        name: "c".to_string()
                                    }),
                                    Node::Token(Token { value: ')' })
                                ],
                                combinator: CombinatorType::Space,
                                disallow_empty: false,
                                explicit: false,
                            }),
                            Node::Multiplier(Multiplier {
                                comma: false,
                                min: 0,
                                max: 0,
                                term: Box::new(Node::Group(Group {
                                    terms: vec![
                                        Node::Multiplier(Multiplier {
                                            comma: false,
                                            min: 0,
                                            max: 1,
                                            term: Box::new(Node::Type(Type {
                                                name: "d".to_string(),
                                                opts: None,
                                            }))
                                        }),
                                        Node::Property(Property {
                                            name: "e".to_string()
                                        }),
                                        Node::Group(Group {
                                            terms: vec![
                                                Node::Token(Token { value: '(' }),
                                                Node::Multiplier(Multiplier {
                                                    comma: false,
                                                    min: 2,
                                                    max: 4,
                                                    term: Box::new(Node::Keyword(Keyword {
                                                        name: "f".to_string()
                                                    }))
                                                }),
                                                Node::Token(Token { value: ')' })
                                            ],
                                            combinator: CombinatorType::Space,
                                            disallow_empty: false,
                                            explicit: false,
                                        })
                                    ],
                                    combinator: CombinatorType::DoubleVerticalLine,
                                    disallow_empty: false,
                                    explicit: true,
                                }))
                            })
                        ],
                        combinator: CombinatorType::DoubleAmpersand,
                        disallow_empty: false,
                        explicit: false,
                    })
                ],
                combinator: CombinatorType::VerticalLine,
                disallow_empty: false,
                explicit: false,
            })
        );
        Ok(())
    }

    #[test]
    fn test_parse_with_range() -> Result<(), SyntaxDefinitionError> {
        let _ = parse("<length-percentage [0,∞]>")?;
        Ok(())
    }

    #[test]
    fn test_parse_quoted_plus() -> Result<(), SyntaxDefinitionError> {
        let result = parse("[ '+' | '-' ]")?;
        assert_eq!(
            result,
            Node::Group(Group {
                terms: vec![
                    Node::String(StringNode {
                        value: "'+'".into()
                    }),
                    Node::String(StringNode {
                        value: "'-'".into()
                    })
                ],
                combinator: CombinatorType::VerticalLine,
                disallow_empty: false,
                explicit: true
            })
        );
        Ok(())
    }

    #[test]
    fn test_parse_function() -> Result<(), SyntaxDefinitionError> {
        let result = parse("rgb() | rgba()")?;
        assert_eq!(
            result,
            Node::Group(Group {
                terms: vec![
                    Node::Group(Group {
                        terms: vec![
                            Node::Function(Function { name: "rgb".into() }),
                            Node::Token(Token { value: ')' })
                        ],
                        combinator: CombinatorType::Space,
                        disallow_empty: false,
                        explicit: false
                    }),
                    Node::Group(Group {
                        terms: vec![
                            Node::Function(Function {
                                name: "rgba".into()
                            }),
                            Node::Token(Token { value: ')' })
                        ],
                        combinator: CombinatorType::Space,
                        disallow_empty: false,
                        explicit: false
                    })
                ],
                combinator: CombinatorType::VerticalLine,
                disallow_empty: false,
                explicit: false
            })
        );
        Ok(())
    }
}
