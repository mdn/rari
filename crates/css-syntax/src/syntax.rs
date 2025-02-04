use std::cmp::{max, min, Ordering};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::sync::LazyLock;

use css_definition_syntax::generate::{self, GenerateOptions};
use css_definition_syntax::parser::{parse, CombinatorType, Multiplier, Node, Type};
use css_definition_syntax::walk::{walk, WalkOptions};
use css_syntax_types::{Css, CssValueType, CssValuesItem};
#[cfg(all(feature = "rari", not(test)))]
use rari_types::globals::data_dir;
use serde::Serialize;

use crate::error::SyntaxError;

static CSS_REF: LazyLock<BTreeMap<String, Css>> = LazyLock::new(|| {
    #[cfg(test)]
    {
        let package_path = std::path::Path::new("package");
        rari_deps::webref_css::update_webref_css(package_path).unwrap();
        let json_str = fs::read_to_string(package_path.join("@webref/css").join("webref_css.json"))
            .expect("no data dir");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    }
    #[cfg(all(not(feature = "rari"), not(test)))]
    {
        let webref_css: &str = include_str!("../@webref/css/webref_css.json");
        serde_json::from_str(webref_css).expect("Failed to parse JSON")
    }
    #[cfg(all(feature = "rari", not(test)))]
    {
        let json_str = fs::read_to_string(data_dir().join("@webref/css").join("webref_css.json"))
            .expect("no data dir");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    }
});

fn flatten_values(values: &'static BTreeMap<String, CssValuesItem>, all: &mut Flattened) {
    for (k, v) in values.iter() {
        if let Some(map) = match v.type_ {
            CssValueType::Type => Some(&mut all.types),
            CssValueType::Function => Some(&mut all.functions),
            CssValueType::Value => Some(&mut all.values),
            CssValueType::Selector => None,
        } {
            map.entry(k).or_insert(v);
        };
        for value in values.values() {
            if let Some(values) = value.values.as_ref() {
                flatten_values(values, all);
            }
        }
    }
}

#[derive(Default, Serialize, Debug)]
pub struct Flattened {
    pub values: BTreeMap<&'static str, &'static CssValuesItem>,
    pub functions: BTreeMap<&'static str, &'static CssValuesItem>,
    pub types: BTreeMap<&'static str, &'static CssValuesItem>,
}

// This relies on the ordered names of CSS_REF.
// "css-values-5" comes after "css-values" and therefore the updated
// overlapping values in "css-values-5" are ignored.
static FLATTENED: LazyLock<Flattened> = LazyLock::new(|| {
    let mut all = Flattened::default();
    let mut entries = CSS_REF.iter().collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        if b.0.ends_with(|c: char| c.is_ascii_digit() || c == '-')
            && b.0
                .trim_end_matches(|c: char| c.is_ascii_digit() || c == '-')
                == a.0
        {
            Ordering::Greater
        } else if a.0.ends_with(|c: char| c.is_ascii_digit() || c == '-')
            && a.0
                .trim_end_matches(|c: char| c.is_ascii_digit() || c == '-')
                == b.0
        {
            Ordering::Less
        } else {
            a.0.cmp(b.0)
        }
    });
    for (_, spec) in entries {
        for (k, item) in spec.values.iter() {
            if let Some(map) = match item.type_ {
                CssValueType::Type => Some(&mut all.types),
                CssValueType::Function => Some(&mut all.functions),
                CssValueType::Value => Some(&mut all.values),
                CssValueType::Selector => None,
            } {
                map.insert(k, item);
            };
            if let Some(values) = item.values.as_ref() {
                flatten_values(values, &mut all);
            }
        }
        for (_, item) in spec.properties.iter() {
            if let Some(values) = item.values.as_ref() {
                flatten_values(values, &mut all);
            }
        }
    }
    all
});

pub enum ItemType {
    Property,
    AtRule,
}

#[derive(Debug, Clone, Copy)]
pub enum CssType<'a> {
    Property(&'a str),
    AtRule(&'a str),
    AtRuleDescriptor(&'a str, &'a str),
    Function(&'a str),
    Type(&'a str),
    ShorthandProperty(&'a str),
}

fn get_specs_for_item<'a>(item_name: &str, item_type: ItemType) -> Vec<&'a str> {
    let mut specs = Vec::new();
    for (name, data) in CSS_REF.iter() {
        let hit = match item_type {
            ItemType::Property => data.properties.contains_key(item_name),
            ItemType::AtRule => data.atrules.contains_key(item_name),
        };
        if hit {
            specs.push(name.as_str())
        }
    }
    specs
}

/// Get the formal syntax for a property from the webref data.
/// # Examples
///
/// ```
/// let color = css_syntax::syntax::get_property_syntax("color");
/// assert_eq!(color, "<color>");
/// ```
///
/// ```
/// let border = css_syntax::syntax::get_property_syntax("border");
/// assert_eq!(border, "<line-width> || <line-style> || <color>");
/// ```
///
/// ```
/// let grid_template_rows = css_syntax::syntax::get_property_syntax("grid-template-rows");
/// assert_eq!(grid_template_rows, "none | <track-list> | <auto-track-list> | subgrid <line-name-list>?");
/// ```
pub fn get_property_syntax(name: &str) -> String {
    // 1) Get all specs which list this property
    let mut specs = get_specs_for_item(name, ItemType::Property);
    // 2) If we have more than one spec, filter out
    //    specs that end "-n" where n is a number
    if specs.len() > 1 {
        specs.retain(|s| {
            !s.rsplit('-')
                .next()
                .map(|s| s.chars().all(char::is_numeric))
                .unwrap_or_default()
        });
    }
    // 3) If we have only one spec, return the syntax it lists
    if specs.len() == 1 {
        return CSS_REF
            .get(specs[0])
            .and_then(|s| s.properties.get(name))
            .and_then(|i| i.value.clone())
            .unwrap_or_default();
    }
    // 4) If we have > 1 spec, assume that:
    // - one of them is the base spec, which defines `values`,
    // - the others define incremental additions as `newValues`

    let (mut syntax, new_syntaxes) = specs.into_iter().fold(
        (String::new(), String::new()),
        |(mut syntax, mut new_syntaxes), spec_name| {
            let base_value = CSS_REF
                .get(spec_name)
                .and_then(|s| s.properties.get(name))
                .and_then(|i| i.value.as_ref());
            let new_values = CSS_REF
                .get(spec_name)
                .and_then(|s| s.properties.get(name))
                .and_then(|i| i.new_values.as_ref());
            if let Some(base_value) = base_value {
                syntax.push_str(base_value);
            }
            if let Some(new_values) = new_values {
                new_syntaxes.push_str(" | ");
                new_syntaxes.push_str(new_values);
            }
            (syntax, new_syntaxes)
        },
    );

    // Concatenate new_values onto values to return a single syntax string
    if !new_syntaxes.is_empty() {
        syntax.push_str(&new_syntaxes);
    }
    syntax
}

/// Get the formal syntax for an at-rule from the webref data.
///
/// Example:
/// ```
/// let media = css_syntax::syntax::get_at_rule_syntax("@media");
/// assert_eq!(media, "@media <media-query-list> { <rule-list> }");
/// ```
pub fn get_at_rule_syntax(name: &str) -> String {
    let specs = get_specs_for_item(name, ItemType::AtRule);

    specs
        .into_iter()
        .find_map(|spec| {
            CSS_REF
                .get(spec)
                .and_then(|s| s.atrules.get(name))
                .and_then(|a| a.value.clone())
        })
        .unwrap_or_default()
}

/// Get the formal syntax for an at-rule descriptor from the webref data.
/// # Example:
/// ```
/// let descriptor = css_syntax::syntax::get_at_rule_descriptor_syntax("width", "@media");
/// assert_eq!(descriptor, "<length>");
/// ```
pub fn get_at_rule_descriptor_syntax(at_rule_descriptor_name: &str, at_rule_name: &str) -> String {
    let specs = get_specs_for_item(at_rule_name, ItemType::AtRule);

    specs
        .into_iter()
        .find_map(|spec| {
            CSS_REF
                .get(spec)
                .and_then(|s| s.atrules.get(at_rule_name))
                .and_then(|a| a.descriptors.get(at_rule_descriptor_name))
                .and_then(|d| d.value.clone())
        })
        .unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Syntax {
    pub name: String,
    pub syntax: String,
}

#[inline]
fn skip(name: &str) -> bool {
    name == "color" || name == "gradient"
}

pub fn get_syntax(typ: CssType) -> Syntax {
    get_syntax_internal(typ, false)
}
fn get_syntax_internal(typ: CssType, top_level: bool) -> Syntax {
    let (name, syntax) = match typ {
        CssType::ShorthandProperty(name) | CssType::Property(name) => {
            let trimmed = name
                .trim_start_matches(['<', '\''])
                .trim_end_matches(['\'', '>']);
            (name.to_string(), get_property_syntax(trimmed))
        }
        CssType::AtRule(name) => (name.to_string(), get_at_rule_syntax(name)),
        CssType::AtRuleDescriptor(name, at_rule_name) => (
            name.to_string(),
            get_at_rule_descriptor_syntax(name, at_rule_name),
        ),
        CssType::Function(name) => {
            let name = format!("{name}()");
            (
                format!("<{name}>"),
                FLATTENED
                    .functions
                    .get(name.as_str())
                    .and_then(|item| item.value.clone())
                    .unwrap_or_default(),
            )
        }
        CssType::Type(name) => {
            let name = name.trim_end_matches("_value");
            if skip(name) && !top_level {
                (format!("<{name}>"), Default::default())
            } else {
                let syntax = FLATTENED
                    .types
                    .get(name)
                    .and_then(|item| item.value.clone())
                    .unwrap_or(
                        FLATTENED
                            .values
                            .get(name)
                            .and_then(|item| item.value.clone())
                            .unwrap_or_default(),
                    );
                let formatted_name = format!("<{name}>");
                let syntax = if name == syntax || formatted_name == syntax {
                    Default::default()
                } else {
                    syntax
                };
                (formatted_name, syntax)
            }
        }
    };

    Syntax { name, syntax }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkedToken {
    Asterisk,
    Plus,
    QuestionMark,
    CurlyBraces,
    HashMark,
    ExclamationPoint,
    Brackets,
    SingleBar,
    DoubleBar,
    DoubleAmpersand,
    Juxtaposition,
}

impl From<CombinatorType> for LinkedToken {
    fn from(value: CombinatorType) -> Self {
        match value {
            CombinatorType::Space => Self::Juxtaposition,
            CombinatorType::DoubleAmpersand => Self::DoubleAmpersand,
            CombinatorType::DoubleVerticalLine => Self::DoubleBar,
            CombinatorType::VerticalLine => Self::SingleBar,
        }
    }
}

impl LinkedToken {
    pub fn fragment(&self) -> &str {
        match self {
            LinkedToken::Asterisk => "asterisk",
            LinkedToken::Plus => "plus",
            LinkedToken::QuestionMark => "question_mark",
            LinkedToken::CurlyBraces => "curly_braces",
            LinkedToken::HashMark => "hash_mark",
            LinkedToken::ExclamationPoint => "exclamation_point_!",
            LinkedToken::Brackets => "brackets",
            LinkedToken::SingleBar => "single_bar",
            LinkedToken::DoubleBar => "double_bar",
            LinkedToken::DoubleAmpersand => "double_ampersand",
            LinkedToken::Juxtaposition => "juxtaposition",
        }
    }
}

#[derive(Debug)]
struct Term {
    pub length: usize,
    pub text: String,
}
pub struct SyntaxRenderer<'a> {
    pub locale_str: &'a str,
    pub value_definition_url: &'a str,
    pub syntax_tooltip: &'a HashMap<LinkedToken, String>,
    pub constituents: HashSet<Node>,
}

impl SyntaxRenderer<'_> {
    pub fn render(&self, output: &mut String, syntax: &Syntax) -> Result<(), SyntaxError> {
        let typ = html_escape::encode_safe(&syntax.name);
        write!(
            output,
            r#"<span class="token property" id="{typ}">{typ} = </span><br/>"#
        )?;

        let ast = parse(&syntax.syntax)?;

        match ast {
            Node::Group(ref group) if group.combinator == CombinatorType::Space => {
                let combinator = group.combinator;
                output.push_str(&self.render_terms(&[ast], combinator)?)
            }
            Node::Group(group) => {
                output.push_str(&self.render_terms(&group.terms, group.combinator)?)
            }
            _ => return Err(SyntaxError::ExpectedGroupNode(ast)),
        };
        Ok(())
    }

    fn generate_linked_token(
        &self,
        out: &mut String,
        linked_token: LinkedToken,
        copy: &str,
        pre_suf: Option<(Option<&str>, Option<&str>)>,
    ) -> Result<(), SyntaxError> {
        let url = &self.value_definition_url;
        let fragment = linked_token.fragment();
        let tooltip = self
            .syntax_tooltip
            .get(&linked_token)
            .map(|s| s.as_str())
            .unwrap_or_default();
        let (prefix, suffix) = match pre_suf {
            Some((pre, suf)) => (pre.unwrap_or_default(), suf.unwrap_or_default()),
            None => ("", ""),
        };
        write!(
            out,
            r#"{prefix}<a href="{url}#{fragment}" title="{tooltip}">{copy}</a>{suffix}"#
        )?;
        Ok(())
    }

    fn render_multiplier(&self, multiplier: &Multiplier) -> Result<String, SyntaxError> {
        let Multiplier {
            comma,
            min,
            max,
            term,
        } = multiplier;
        let mut out = String::new();
        write!(&mut out, "{}", self.render_term(term)?.text)?;
        if *comma {
            self.generate_linked_token(&mut out, LinkedToken::HashMark, "#", None)?
        }
        match (min, max) {
            (0, 0) if *comma => {
                self.generate_linked_token(&mut out, LinkedToken::QuestionMark, "?", None)?
            }
            (0, 0) => self.generate_linked_token(&mut out, LinkedToken::Asterisk, "*", None)?,
            (0, 1) => self.generate_linked_token(&mut out, LinkedToken::QuestionMark, "?", None)?,
            (1, 0) if *comma => (),
            (1, 0) => self.generate_linked_token(&mut out, LinkedToken::Plus, "+", None)?,
            (1, 1) => (),
            _ => {
                let copy = match (min, max) {
                    (min, max) if min == max => format!("{{{min}}}"),
                    (min, 0) => format!("{{{min},}}"),
                    (min, max) => format!("{{{min},{max}}}"),
                };
                self.generate_linked_token(&mut out, LinkedToken::CurlyBraces, &copy, None)?;
            }
        };
        Ok(out)
    }

    fn render_node(&self, name: &str, node: &Node) -> Result<String, SyntaxError> {
        let out = match node {
            Node::Multiplier(multiplier) => self.render_multiplier(multiplier)?,
            Node::Token(_) if name == ")" => r#"<span class="token function">)</span>"#.into(),
            Node::Property(_) => {
                let encoded = html_escape::encode_safe(name);
                if name.starts_with("<'") && name.ends_with("'>") {
                    let slug = &name[2..name.len() - 2];
                    format!(
                        r#"<a href="/{}/docs/Web/CSS/{slug}"><span class="token property">{encoded}</span></a>"#,
                        self.locale_str
                    )
                } else {
                    format!(r#"<span class="token property">{encoded}</span>"#)
                }
            }
            Node::Type(typ) => {
                let encoded = html_escape::encode_safe(name);
                let slug = match name {
                    "<color>" => "color_value",
                    "<position>" => "position_value",
                    name if name.starts_with('<') && name.ends_with('>') => {
                        &name[1..name.find(" [").or(name.find('[')).unwrap_or(name.len() - 1)]
                    }
                    name => &name[0..name.find(" [").or(name.find('[')).unwrap_or(name.len())],
                };

                if !skip(slug)
                    && (self.constituents.contains(node)
                        || self.constituents.contains(&Node::Type(Type {
                            name: typ.name.clone(),
                            opts: None,
                        })))
                {
                    // FIXME: this should have the class type but to be compatible we use property
                    format!(r#"<span class="token property">{encoded}</span>"#,)
                } else {
                    // FIXME: this should have the class type but to be compatible we use property
                    format!(
                        r#"<a href="/{}/docs/Web/CSS/{slug}"><span class="token property">{encoded}</span></a>"#,
                        self.locale_str
                    )
                }
            }
            Node::Function(_) => {
                let encoded = html_escape::encode_safe(name);
                format!(r#"<span class="token function">{encoded}</span>"#)
            }
            Node::Keyword(_) => {
                let encoded = html_escape::encode_safe(name);
                format!(r#"<span class="token keyword">{encoded}</span>"#)
            }
            Node::Group(group) => {
                let mut opening_bracket_link = String::new();
                self.generate_linked_token(
                    &mut opening_bracket_link,
                    LinkedToken::Brackets,
                    "[",
                    Some((None, Some(" "))),
                )?;
                let mut closing_bracket_link = String::new();
                self.generate_linked_token(
                    &mut closing_bracket_link,
                    LinkedToken::Brackets,
                    "]",
                    Some((Some(" "), None)),
                )?;

                let mut out = name
                    .replace("[ ", &opening_bracket_link)
                    .replace(" ]", &closing_bracket_link);
                // TODO: remove
                if group.combinator != CombinatorType::Space {
                    let mut combinator_link = String::new();
                    self.generate_linked_token(
                        &mut combinator_link,
                        group.combinator.into(),
                        group.combinator.as_str_compact(),
                        Some((Some(" "), Some(" "))),
                    )?;
                    out = out.replace(group.combinator.as_str(), &combinator_link);
                }
                out
            }
            _ => name.to_string(),
        };
        Ok(out)
    }

    fn render_term(&self, term: &Node) -> Result<Term, SyntaxError> {
        let length = generate::generate(term, Default::default())?
            .chars()
            .count();
        let text = generate::generate(
            term,
            GenerateOptions {
                decorate: &|name, node| self.render_node(&name, node).unwrap_or(name),
                ..Default::default()
            },
        )?;
        Ok(Term { length, text })
    }

    fn render_terms(
        &self,
        terms: &[Node],
        combinator: CombinatorType,
    ) -> Result<String, SyntaxError> {
        let terms = terms
            .iter()
            .map(|node| self.render_term(node))
            .collect::<Result<Vec<_>, SyntaxError>>()?;

        let max_line_len = 50;
        let max_term_len = min(
            terms.iter().map(|i| i.length).max().unwrap_or_default(),
            max_line_len,
        ) as i32;

        let len = terms.len();
        terms.into_iter().enumerate().try_fold(
            String::new(),
            |mut output, (i, Term { text, length })| {
                let space_count = max(2, max_term_len + 2 - length as i32);
                let combinator_text = if combinator == CombinatorType::Space {
                    "".to_string()
                } else if i < len - 1 {
                    let linked_token = LinkedToken::from(combinator);
                    format!(
                        r#"<a href="{}#{}" title="{}">{}</a>"#,
                        self.value_definition_url,
                        linked_token.fragment(),
                        self.syntax_tooltip
                            .get(&linked_token)
                            .map(|s| s.as_str())
                            .unwrap_or_default(),
                        combinator.as_str_compact()
                    )
                } else {
                    Default::default()
                };
                write!(
                    output,
                    "  {text}{}{combinator_text}<br/>",
                    " ".repeat(space_count as usize)
                )
                .map_err(|_| SyntaxError::IoError)?;

                Ok::<String, SyntaxError>(output)
            },
        )
    }
    fn get_constituent_syntaxes(&mut self, syntax: Syntax) -> Result<Vec<Syntax>, SyntaxError> {
        let mut all_constituents = vec![];

        let mut last_len: usize;
        let mut last_syntax_len = 0;

        let mut constituent_syntaxes: Vec<Syntax> = vec![syntax];

        loop {
            last_len = all_constituents.len();
            get_nodes_for_syntaxes(
                &constituent_syntaxes[last_syntax_len..],
                &mut all_constituents,
            )?;

            if all_constituents.len() <= last_len {
                break;
            }

            last_syntax_len = constituent_syntaxes.len();

            for constituent in all_constituents[last_len..].iter_mut() {
                if let Some(constituent_entry) = match &mut constituent.node {
                    Node::Type(typ) if typ.name.ends_with("()") => {
                        let syntax = get_syntax(CssType::Function(&typ.name[..typ.name.len() - 2]));
                        Some(syntax)
                    }
                    Node::Type(typ) => {
                        let syntax = get_syntax(CssType::Type(&typ.name));
                        typ.opts = None;
                        Some(syntax)
                    }
                    Node::Property(property) => {
                        let mut syntax = get_syntax(CssType::Property(&property.name));
                        syntax.name = format!("<{}>", syntax.name);
                        Some(syntax)
                    }
                    // Node::Function(function) => Some(get_syntax(CssType::Function(&function.name))),
                    Node::AtKeyword(at_keyword) => {
                        Some(get_syntax(CssType::AtRule(&at_keyword.name)))
                    }
                    _ => None,
                } {
                    if !constituent_entry.syntax.is_empty()
                        && !constituent_syntaxes.contains(&constituent_entry)
                    {
                        constituent.syntax_used = true;
                        constituent_syntaxes.push(constituent_entry)
                    }
                }
            }
        }
        self.constituents
            .extend(all_constituents.into_iter().filter_map(|constituent| {
                if constituent.syntax_used {
                    Some(constituent.node)
                } else {
                    None
                }
            }));
        Ok(constituent_syntaxes)
    }
}

#[derive(Debug)]
struct Constituent {
    node: Node,
    syntax_used: bool,
}

impl From<Node> for Constituent {
    fn from(node: Node) -> Self {
        Constituent {
            node,
            syntax_used: false,
        }
    }
}

pub fn write_formal_syntax_from_syntax(
    syntax_str: impl Into<String>,
    locale_str: &str,
    value_definition_url: &str,
    syntax_tooltip: &'_ HashMap<LinkedToken, String>,
) -> Result<String, SyntaxError> {
    let syntax_str = syntax_str.into();
    let (name, syntax, skip_first) = if let Some((name, syntax)) = syntax_str.split_once("=") {
        (name, syntax.trim().to_string(), false)
    } else {
        ("dummy", syntax_str, true)
    };
    let syntax = Syntax {
        name: name.trim().to_string(),
        syntax,
    };
    write_formal_syntax_internal(
        syntax,
        locale_str,
        value_definition_url,
        syntax_tooltip,
        skip_first,
    )
}

pub fn write_formal_syntax(
    css: CssType,
    locale_str: &str,
    value_definition_url: &str,
    syntax_tooltip: &'_ HashMap<LinkedToken, String>,
) -> Result<String, SyntaxError> {
    let syntax: Syntax = get_syntax_internal(css, true);
    if syntax.syntax.is_empty() {
        return Err(SyntaxError::NoSyntaxFound);
    }
    write_formal_syntax_internal(
        syntax,
        locale_str,
        value_definition_url,
        syntax_tooltip,
        false,
    )
}

fn write_formal_syntax_internal(
    syntax: Syntax,
    locale_str: &str,
    value_definition_url: &str,
    syntax_tooltip: &'_ HashMap<LinkedToken, String>,
    skip_first: bool,
) -> Result<String, SyntaxError> {
    let mut renderer = SyntaxRenderer {
        locale_str,
        value_definition_url,
        syntax_tooltip,
        constituents: Default::default(),
    };
    let mut out = String::new();
    write!(out, r#"<pre class="notranslate">"#)?;
    let constituents = renderer.get_constituent_syntaxes(syntax)?;

    for constituent in constituents.iter().skip(if skip_first { 1 } else { 0 }) {
        renderer.render(&mut out, constituent)?;
        out.push_str("<br/>");
    }

    out.push_str("</pre>");
    Ok(out)
}

fn get_nodes_for_syntaxes(
    syntaxes: &[Syntax],
    constituents: &mut Vec<Constituent>,
) -> Result<(), SyntaxError> {
    for syntax in syntaxes {
        if syntax.syntax.is_empty() {
            continue;
        }
        let ast = parse(&syntax.syntax);

        walk(
            &ast?,
            &WalkOptions::<Vec<Constituent>> {
                enter: |node: &Node, context: &mut Vec<Constituent>| {
                    if !skip(node.str_name())
                        && !context.iter().any(|constituent| constituent.node == *node)
                    {
                        context.push(node.clone().into())
                    }
                    Ok(())
                },
                ..Default::default()
            },
            constituents,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    static TOOLTIPS: LazyLock<HashMap<LinkedToken, String>> = LazyLock::new(|| {
        [(LinkedToken::Asterisk, "Asterisk: the entity may occur zero, one or several times".to_string()),
    (LinkedToken::Plus, "Plus: the entity may occur one or several times".to_string()),
    (LinkedToken::QuestionMark, "Question mark: the entity is optional".to_string()),
    (LinkedToken::CurlyBraces, "Curly braces: encloses two integers defining the minimal and maximal numbers of occurrences of the entity, or a single integer defining the exact number required".to_string()),
    (LinkedToken::HashMark, "Hash mark: the entity is repeated one or several times, each occurrence separated by a comma".to_string()),
    (LinkedToken::ExclamationPoint,"Exclamation point: the group must produce at least one value".to_string()),
    (LinkedToken::Brackets, "Brackets: enclose several entities, combinators, and multipliers to transform them as a single component".to_string()),
    (LinkedToken::SingleBar, "Single bar: exactly one of the entities must be present".to_string()),
    (LinkedToken::DoubleBar, "Double bar: one or several of the entities must be present, in any order".to_string()),
    (LinkedToken::DoubleAmpersand, "Double ampersand: all of the entities must be present, in any order".to_string())].into_iter().collect()
    });

    #[test]
    fn test_get_syntax_color_property_color() {
        let Syntax { name, syntax } = get_syntax_internal(CssType::Property("color"), true);
        assert_eq!(name, "color");
        assert_eq!(syntax, "<color>");
    }
    #[test]
    fn test_get_syntax_color_property_content_visibility() {
        let Syntax { name, syntax } = get_syntax(CssType::Property("content-visibility"));
        assert_eq!(name, "content-visibility");
        assert_eq!(syntax, "visible | auto | hidden");
    }
    #[test]
    fn test_get_syntax_length_type() {
        let Syntax { name, syntax } = get_syntax(CssType::Type("length"));
        assert_eq!(name, "<length>");
        assert_eq!(syntax, "");
    }
    #[test]
    fn test_get_syntax_color_type() {
        let Syntax { name, syntax } = get_syntax_internal(CssType::Type("color_value"), true);
        assert_eq!(name, "<color>");
        assert_eq!(syntax, "<color-base> | currentColor | <system-color>");
    }
    #[test]
    fn test_get_syntax_minmax_function() {
        let Syntax { name, syntax } = get_syntax(CssType::Function("minmax"));
        assert_eq!(name, "<minmax()>");
        assert_eq!(syntax, "minmax(min, max)");
    }
    #[test]
    fn test_get_syntax_sin_function() {
        let Syntax { name, syntax } = get_syntax(CssType::Function("sin"));
        assert_eq!(name, "<sin()>");
        assert_eq!(syntax, "sin( <calc-sum> )");
    }
    #[test]
    fn test_get_syntax_media_at_rule() {
        let Syntax { name, syntax } = get_syntax(CssType::AtRule("@media"));
        assert_eq!(name, "@media");
        assert_eq!(syntax, "@media <media-query-list> { <rule-list> }");
    }
    #[test]
    fn test_get_syntax_padding_property() {
        let Syntax { name, syntax } = get_syntax(CssType::Property("padding"));
        assert_eq!(name, "padding");
        assert_eq!(syntax, "<'padding-top'>{1,4}");
    }
    #[test]
    fn test_get_syntax_gradient_type() {
        let Syntax { name, syntax } = get_syntax_internal(CssType::Type("gradient"), true);
        assert_eq!(name, "<gradient>");
        assert_eq!(syntax, "<linear-gradient()> | <repeating-linear-gradient()> | <radial-gradient()> | <repeating-radial-gradient()>");
    }

    #[test]
    fn test_render_terms() -> Result<(), SyntaxError> {
        let renderer = SyntaxRenderer {
            locale_str: "en-US",
            value_definition_url: "/en-US/docs/Web/CSS/Value_definition_syntax",
            syntax_tooltip: &TOOLTIPS,
            constituents: Default::default(),
        };
        let Syntax { name: _, syntax } = get_syntax_internal(CssType::Type("color_value"), true);
        if let Node::Group(group) = parse(&syntax)? {
            let rendered = renderer.render_terms(&group.terms, group.combinator)?;
            assert_eq!(rendered, "  <a href=\"/en-US/docs/Web/CSS/color-base\"><span class=\"token property\">&lt;color-base&gt;</span></a>    <a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <span class=\"token keyword\">currentColor</span>    <a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/system-color\"><span class=\"token property\">&lt;system-color&gt;</span></a>  <br/>");
        } else {
            panic!("no group node")
        }
        Ok(())
    }

    #[test]
    fn test_render_node() -> Result<(), SyntaxError> {
        let expected = "<pre class=\"notranslate\"><span class=\"token property\" id=\"padding\">padding = </span><br/>  <a href=\"/en-US/docs/Web/CSS/padding-top\"><span class=\"token property\">&lt;&#x27;padding-top&#x27;&gt;</span></a><a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#curly_braces\" title=\"Curly braces: encloses two integers defining the minimal and maximal numbers of occurrences of the entity, or a single integer defining the exact number required\">{1,4}</a>  <br/><br/><span class=\"token property\" id=\"&lt;padding-top&gt;\">&lt;padding-top&gt; = </span><br/>  <span class=\"token property\">&lt;length-percentage [0,∞]&gt;</span>  <br/><br/><span class=\"token property\" id=\"&lt;length-percentage&gt;\">&lt;length-percentage&gt; = </span><br/>  <a href=\"/en-US/docs/Web/CSS/length\"><span class=\"token property\">&lt;length&gt;</span></a>      <a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/percentage\"><span class=\"token property\">&lt;percentage&gt;</span></a>  <br/><br/></pre>";
        let result = write_formal_syntax(
            CssType::Property("padding"),
            "en-US",
            "/en-US/docs/Web/CSS/Value_definition_syntax",
            &TOOLTIPS,
        )?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_render_function() -> Result<(), SyntaxError> {
        let expected = "<pre class=\"notranslate\"><span class=\"token property\" id=\"&lt;hue-rotate()&gt;\">&lt;hue-rotate()&gt; = </span><br/>  <span class=\"token function\">hue-rotate(</span> <a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#brackets\" title=\"Brackets: enclose several entities, combinators, and multipliers to transform them as a single component\">[</a> <a href=\"/en-US/docs/Web/CSS/angle\"><span class=\"token property\">&lt;angle&gt;</span></a> <a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a> <a href=\"/en-US/docs/Web/CSS/zero\"><span class=\"token property\">&lt;zero&gt;</span></a> <a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#brackets\" title=\"Brackets: enclose several entities, combinators, and multipliers to transform them as a single component\">]</a><a href=\"/en-US/docs/Web/CSS/Value_definition_syntax#question_mark\" title=\"Question mark: the entity is optional\">?</a> <span class=\"token function\">)</span>  <br/><br/></pre>";
        let result = write_formal_syntax(
            CssType::Function("hue-rotate"),
            "en-US",
            "/en-US/docs/Web/CSS/Value_definition_syntax",
            &TOOLTIPS,
        )?;
        assert_eq!(result, expected);
        Ok(())
    }
}
