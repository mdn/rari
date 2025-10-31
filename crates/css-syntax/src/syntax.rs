use std::cmp::{max, min};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write;
#[cfg(any(feature = "rari", test))]
use std::fs;
use std::sync::LazyLock;

use css_definition_syntax::generate::{self, GenerateOptions};
use css_definition_syntax::parser::{parse, CombinatorType, Multiplier, Node, Type};
use css_definition_syntax::walk::{walk, WalkOptions};
use css_syntax_types::{CssValuesItem, SpecLink, WebrefCss};
#[cfg(all(feature = "rari", not(any(feature = "doctest", test))))]
use rari_types::globals::data_dir;
use serde::Serialize;

use crate::error::SyntaxError;

static CSS_REF: LazyLock<WebrefCss> = LazyLock::new(|| {
    #[cfg(any(feature = "doctest", test))]
    {
        let package_path = std::path::Path::new("package");
        rari_deps::webref_css::update_webref_css(package_path).unwrap();
        let json_str = fs::read_to_string(
            package_path
                .join("@webref")
                .join("css")
                .join("webref_css.json"),
        )
        .expect("no data dir");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    }
    #[cfg(all(not(feature = "rari"), not(any(feature = "doctest", test))))]
    {
        let webref_css: &str = include_str!("../@webref/css/webref_css.json");
        serde_json::from_str(webref_css).expect("Failed to parse JSON")
    }
    #[cfg(all(feature = "rari", not(any(feature = "doctest", test))))]
    {
        let json_str = fs::read_to_string(data_dir().join("@webref/css").join("webref_css.json"))
            .expect("no data dir");
        serde_json::from_str(&json_str).expect("Failed to parse JSON")
    }
});

pub type ItemAndHref = (&'static CssValuesItem, Option<&'static SpecLink>);

#[derive(Default, Serialize, Debug)]
pub struct Flattened {
    pub values: BTreeMap<&'static str, ItemAndHref>,
    pub functions: BTreeMap<&'static str, ItemAndHref>,
    pub types: BTreeMap<&'static str, ItemAndHref>,
}

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

/// Get the formal syntax for a property from the webref data.
/// # Examples
///
/// ```
/// let color = css_syntax::syntax::get_property_syntax("color");
/// assert_eq!(color.syntax, "<color>");
/// ```
///
/// ```
/// let border = css_syntax::syntax::get_property_syntax("border");
/// assert_eq!(border.syntax, "<line-width> || <line-style> || <color>");
/// ```
///
/// ```
/// let grid_template_rows = css_syntax::syntax::get_property_syntax("grid-template-rows");
/// assert_eq!(grid_template_rows.syntax, "none | <track-list> | <auto-track-list> | subgrid <line-name-list>?");
/// ```
pub fn get_property_syntax(name: &str, browser_compat: Option<&str>) -> Syntax {
    // TODO: proper scoping via for
    if let Some(scoped) = CSS_REF
        .properties
        .get(browser_compat.unwrap_or("__no_for__")) && let Some(property) = scoped.get(name)
    {
        return Syntax {
            syntax: property.syntax.clone().unwrap_or_default(),
            specs: property.spec_link.as_ref().map(|s| vec![s]),
        };
    }
    Syntax::default()
}

/// Get the formal syntax for an at-rule from the webref data.
pub fn get_at_rule_syntax(name: &str) -> Syntax {
    // TODO: proper scoping via for
    if let Some(property) = CSS_REF.atrules.get("__no_for__").unwrap().get(name) {
        return Syntax {
            syntax: property.syntax.clone().unwrap_or_default(),
            specs: property.spec_link.as_ref().map(|s| vec![s]),
        };
    }
    Syntax::default()
}

/// Get the formal syntax for an at-rule descriptor from the webref data.
pub fn get_at_rule_descriptor_syntax(at_rule_descriptor_name: &str, at_rule_name: &str) -> Syntax {
    // TODO: proper scoping via for
    if let Some(at_rule) = CSS_REF.atrules.get("__no_for__").unwrap().get(at_rule_name) {
        if let Some(at_rule_descriptor) = at_rule.descriptors.get(at_rule_descriptor_name) {
            return Syntax {
                syntax: at_rule_descriptor.syntax.clone().unwrap_or_default(),
                specs: at_rule_descriptor.spec_link.as_ref().map(|s| vec![s]),
            };
        }
    }
    Syntax::default()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxLine {
    pub name: String,
    pub syntax: String,
    pub specs: Option<Vec<&'static SpecLink>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Syntax {
    pub syntax: String,
    pub specs: Option<Vec<&'static SpecLink>>,
}

impl Syntax {
    pub fn to_syntax_line(self, name: impl Into<String>) -> SyntaxLine {
        SyntaxLine {
            name: name.into(),
            syntax: self.syntax,
            specs: self.specs,
        }
    }
}

#[inline]
fn skip(name: &str) -> bool {
    name == "color" || name == "gradient"
}

pub fn get_syntax(typ: CssType, browser_compat: Option<&str>) -> SyntaxLine {
    get_syntax_internal(typ, browser_compat, false)
}

fn get_scoped_syntax(typ: CssType, browser_compat: Option<&str>) -> SyntaxLine {
    match typ {
        CssType::Property(_) => todo!(),
        CssType::AtRule(_) => todo!(),
        CssType::AtRuleDescriptor(_, _) => todo!(),
        CssType::Function(_) => todo!(),
        CssType::Type(_) => todo!(),
        CssType::ShorthandProperty(_) => todo!(),
    }
}

fn get_syntax_internal(typ: CssType, browser_compat: Option<&str>, top_level: bool) -> SyntaxLine {
    let scope_key = browser_compat.unwrap_or("__no_for__");
    match typ {
        CssType::ShorthandProperty(name) | CssType::Property(name) => {
            let trimmed = name
                .trim_start_matches(['<', '\''])
                .trim_end_matches(['\'', '>']);
            get_property_syntax(trimmed).to_syntax_line(name)
        }
        CssType::Type(name) => {
            let name = name.trim_end_matches("_value");
            if skip(name) && !top_level {
                Syntax::default().to_syntax_line(format!("<{name}>"))
            } else if let Some(t) = CSS_REF.types.get(name) {
                if let Some(syntax) = &t.syntax {
                    Syntax {
                        syntax: syntax.clone(),
                        specs: t.spec_link.as_ref().map(|s| vec![s]),
                    }
                    .to_syntax_line(format!("<{name}>"))
                } else {
                    Syntax::default().to_syntax_line(format!("<{name}>"))
                }
            } else {
                Syntax::default().to_syntax_line(format!("<{name}>"))
            }
        }
        CssType::Function(name) => {
            let name = format!("{name}()");
            if let Some(t) = CSS_REF.functions.get(&name) {
                if let Some(syntax) = &t.syntax {
                    return Syntax {
                        syntax: syntax.clone(),
                        specs: t.spec_link.as_ref().map(|s| vec![s]),
                    }
                    .to_syntax_line(format!("<{name}>"));
                }
            }
            Syntax::default().to_syntax_line(format!("<{name}>"))
        }
        CssType::AtRule(name) => get_at_rule_syntax(name).to_syntax_line(name),
        CssType::AtRuleDescriptor(name, at_rule_name) => {
            get_at_rule_descriptor_syntax(name, at_rule_name).to_syntax_line(name)
        }
    }
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
    pub fn render(&self, output: &mut String, syntax: &SyntaxLine) -> Result<(), SyntaxError> {
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
    fn get_constituent_syntaxes(
        &mut self,
        syntax: SyntaxLine,
    ) -> Result<Vec<SyntaxLine>, SyntaxError> {
        let mut all_constituents = vec![];

        let mut last_len: usize;
        let mut last_syntax_len = 0;

        let mut constituent_syntaxes: Vec<SyntaxLine> = vec![syntax];

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

pub enum SyntaxInput<'a> {
    SyntaxString(&'a str),
    Css(CssType<'a>),
}

pub fn render_formal_syntax(
    syntax: SyntaxInput,
    browser_compat: Option<&str>,
    locale_str: &str,
    value_definition_url: &str,
    syntax_tooltip: &HashMap<LinkedToken, String>,
    sources_prefix: Option<&str>,
) -> Result<String, SyntaxError> {
    let (syntax, skip_first) = match syntax {
        SyntaxInput::SyntaxString(syntax_str) => {
            let (name, syntax, skip_first) =
                if let Some((name, syntax)) = syntax_str.split_once("=") {
                    (name, syntax.trim().to_string(), false)
                } else {
                    ("dummy", syntax_str.into(), true)
                };
            (
                SyntaxLine {
                    name: name.trim().to_string(),
                    syntax,
                    specs: None,
                },
                skip_first,
            )
        }
        SyntaxInput::Css(css) => {
            let syntax: SyntaxLine = get_syntax_internal(css, browser_compat, true);
            if syntax.syntax.is_empty() {
                return Err(SyntaxError::NoSyntaxFound);
            }
            (syntax, false)
        }
    };
    render_formal_syntax_internal(
        syntax,
        locale_str,
        value_definition_url,
        syntax_tooltip,
        sources_prefix,
        skip_first,
    )
}

fn render_formal_syntax_internal(
    syntax: SyntaxLine,
    locale_str: &str,
    value_definition_url: &str,
    syntax_tooltip: &'_ HashMap<LinkedToken, String>,
    sources_prefix: Option<&str>,
    skip_first: bool,
) -> Result<String, SyntaxError> {
    let mut renderer = SyntaxRenderer {
        locale_str,
        value_definition_url,
        syntax_tooltip,
        constituents: Default::default(),
    };
    let mut out = String::new();
    write!(out, r#"<pre class="notranslate css-formal-syntax">"#)?;
    let mut constituents = renderer.get_constituent_syntaxes(syntax)?;

    for (i, constituent) in constituents
        .iter()
        .skip(if skip_first { 1 } else { 0 })
        .enumerate()
    {
        if i > 0 {
            out.push_str("<br/>");
        }
        renderer.render(&mut out, constituent)?;
    }

    let specs = constituents.iter_mut().fold(vec![], |mut acc, s| {
        if let Some(spec) = s.specs.take() {
            if !acc.contains(&spec) {
                acc.push(spec)
            }
        }
        acc
    });

    out.push_str("</pre>");
    if !specs.is_empty() {
        out.push_str("<footer>");
        let mut unique_spec_links = BTreeSet::new();

        for spec in specs.iter() {
            if let Some(spec) = spec.first() {
                let mut url_without_fragment = spec.url.clone();
                url_without_fragment.set_fragment(None);
                unique_spec_links.insert(SpecLink {
                    url: url_without_fragment,
                    title: spec.title.clone(),
                });
            }
        }

        if let Some(sources_prefix) = sources_prefix {
            // The sources_prefix l10n value has a replacement for the spec links, denoted by `{ $specs }`.
            // Replace this placeholder by the comma-separated list of specs
            out.push_str(
                &sources_prefix.replace(
                    "{ $specs }",
                    unique_spec_links
                        .iter()
                        .map(|spec| {
                            format!(r#"<a href="{}">{}</a>"#, spec.url.as_str(), spec.title)
                        })
                        .collect::<Vec<String>>()
                        .join(", ")
                        .as_str(),
                ),
            );
        }
        out.push_str("</footer>");
    }
    Ok(out)
}

fn get_nodes_for_syntaxes(
    syntaxes: &[SyntaxLine],
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
        let SyntaxLine { name, syntax, .. } =
            get_syntax_internal(CssType::Property("color"), None, true);
        assert_eq!(name, "color");
        assert_eq!(syntax, "<color>");
    }
    #[test]
    fn test_get_syntax_color_property_content_visibility() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::Property("content-visibility"));
        assert_eq!(name, "content-visibility");
        assert_eq!(syntax, "visible | auto | hidden");
    }
    #[test]
    fn test_get_syntax_length_type() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::Type("length"));
        assert_eq!(name, "<length>");
        assert_eq!(syntax, "");
    }
    #[test]
    fn test_get_syntax_color_type() {
        let SyntaxLine { name, syntax, .. } =
            get_syntax_internal(CssType::Type("color_value"), None, true);
        assert_eq!(name, "<color>");
        assert_eq!(syntax, "<color-base> | currentColor | <system-color> | <contrast-color()> | <device-cmyk()> | <light-dark()>");
    }
    #[test]
    fn test_get_syntax_minmax_function() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::Function("minmax"));
        assert_eq!(name, "<minmax()>");
        assert_eq!(syntax, "minmax(min, max)");
    }
    #[test]
    fn test_get_syntax_sin_function() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::Function("sin"));
        assert_eq!(name, "<sin()>");
        assert_eq!(syntax, "sin( <calc-sum> )");
    }
    #[test]
    fn test_get_syntax_media_at_rule() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::AtRule("@media"));
        assert_eq!(name, "@media");
        assert_eq!(syntax, "@media <media-query-list> { <rule-list> }");
    }
    #[test]
    fn test_get_syntax_padding_property() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::Property("padding"));
        assert_eq!(name, "padding");
        assert_eq!(syntax, "<'padding-top'>{1,4}");
    }
    #[test]
    fn test_get_syntax_gradient_type() {
        let SyntaxLine { name, syntax, .. } = get_syntax_internal(CssType::Type("gradient"), true);
        assert_eq!(name, "<gradient>");
        assert_eq!(syntax, "[ <linear-gradient()> | <repeating-linear-gradient()> | <radial-gradient()> | <repeating-radial-gradient()> | <conic-gradient()> | <repeating-conic-gradient()> ]");
    }

    #[test]
    fn test_get_atrule_descriptor_counter_style_additive_symbols() {
        let SyntaxLine { name, syntax, .. } = get_syntax(CssType::AtRuleDescriptor(
            "additive-symbols",
            "@counter-style",
        ));
        assert_eq!(name, "additive-symbols");
        assert_eq!(syntax, "[ <integer [0,∞]> && <symbol> ]#");
    }

    #[test]
    fn test_render_terms() -> Result<(), SyntaxError> {
        let renderer = SyntaxRenderer {
            locale_str: "en-US",
            value_definition_url:
                "/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax",
            syntax_tooltip: &TOOLTIPS,
            constituents: Default::default(),
        };
        let SyntaxLine {
            name: _, syntax, ..
        } = get_syntax_internal(CssType::Type("color_value"), true);
        if let Node::Group(group) = parse(&syntax)? {
            let rendered = renderer.render_terms(&group.terms, group.combinator)?;
            assert_eq!(rendered, "  <a href=\"/en-US/docs/Web/CSS/color-base\"><span class=\"token property\">&lt;color-base&gt;</span></a>        <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <span class=\"token keyword\">currentColor</span>        <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/system-color\"><span class=\"token property\">&lt;system-color&gt;</span></a>      <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/contrast-color()\"><span class=\"token property\">&lt;contrast-color()&gt;</span></a>  <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/device-cmyk()\"><span class=\"token property\">&lt;device-cmyk()&gt;</span></a>     <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/light-dark()\"><span class=\"token property\">&lt;light-dark()&gt;</span></a>      <br/>");
        } else {
            panic!("no group node")
        }
        Ok(())
    }

    #[test]
    fn test_render_node() -> Result<(), SyntaxError> {
        let expected = "<pre class=\"notranslate css-formal-syntax\"><span class=\"token property\" id=\"padding\">padding = </span><br/>  <a href=\"/en-US/docs/Web/CSS/padding-top\"><span class=\"token property\">&lt;&#x27;padding-top&#x27;&gt;</span></a><a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#curly_braces\" title=\"Curly braces: encloses two integers defining the minimal and maximal numbers of occurrences of the entity, or a single integer defining the exact number required\">{1,4}</a>  <br/><br/><span class=\"token property\" id=\"&lt;padding-top&gt;\">&lt;padding-top&gt; = </span><br/>  <span class=\"token property\">&lt;length-percentage [0,∞]&gt;</span>  <br/><br/><span class=\"token property\" id=\"&lt;length-percentage&gt;\">&lt;length-percentage&gt; = </span><br/>  <a href=\"/en-US/docs/Web/CSS/length\"><span class=\"token property\">&lt;length&gt;</span></a>      <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a><br/>  <a href=\"/en-US/docs/Web/CSS/percentage\"><span class=\"token property\">&lt;percentage&gt;</span></a>  <br/></pre><footer></footer>";
        let result = render_formal_syntax(
            SyntaxInput::Css(CssType::Property("padding")),
            &[],
            "en-US",
            "/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax",
            &TOOLTIPS,
            None,
        )?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_render_function() -> Result<(), SyntaxError> {
        let expected = "<pre class=\"notranslate css-formal-syntax\"><span class=\"token property\" id=\"&lt;hue-rotate()&gt;\">&lt;hue-rotate()&gt; = </span><br/>  <span class=\"token function\">hue-rotate(</span> <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#brackets\" title=\"Brackets: enclose several entities, combinators, and multipliers to transform them as a single component\">[</a> <a href=\"/en-US/docs/Web/CSS/angle\"><span class=\"token property\">&lt;angle&gt;</span></a> <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#single_bar\" title=\"Single bar: exactly one of the entities must be present\">|</a> <a href=\"/en-US/docs/Web/CSS/zero\"><span class=\"token property\">&lt;zero&gt;</span></a> <a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#brackets\" title=\"Brackets: enclose several entities, combinators, and multipliers to transform them as a single component\">]</a><a href=\"/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax#question_mark\" title=\"Question mark: the entity is optional\">?</a> <span class=\"token function\">)</span>  <br/></pre><footer></footer>";
        let result = render_formal_syntax(
            SyntaxInput::Css(CssType::Function("hue-rotate")),
            &[],
            "en-US",
            "/en-US/docs/Web/CSS/CSS_values_and_units/Value_definition_syntax",
            &TOOLTIPS,
            None,
        )?;
        assert_eq!(result, expected);
        Ok(())
    }
}
