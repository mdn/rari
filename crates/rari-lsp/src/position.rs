use std::sync::LazyLock;

use tower_lsp_server::lsp_types::Position;

use crate::lsp::Document;

#[derive(Clone, Debug)]
pub enum Element {
    Link {
        link: String,
        start: Position,
        end: Position,
    },
}

pub(crate) fn retrieve_element_at_position(
    doc: &mut Document,
    parser: &mut tree_sitter_md::MarkdownParser,
    cursor_line: usize,
    cursor_character: usize,
) -> Option<Element> {
    let document_content = doc.full.get_content(None);
    doc.md_tree = parser.parse(document_content.as_bytes(), doc.md_tree.as_ref());
    let tree = doc.md_tree.as_ref()?;

    let mut query_cursor = tree.walk();
    while query_cursor
        .goto_first_child_for_point(tree_sitter::Point {
            row: cursor_line,
            column: cursor_character,
        })
        .is_some()
    {}
    let mut node = query_cursor.node();

    if node.grammar_name() == node.kind() && node.kind().len() == 1 {
        if let Some(parent) = node.parent() {
            node = parent
        }
    }
    match node.grammar_name() {
        "link_destination" => {
            let start_position = node.start_position();
            let start = Position {
                line: start_position.row as u32,
                character: start_position.column as u32,
            };
            let end_position = node.end_position();
            let end = Position {
                line: end_position.row as u32,
                character: end_position.column as u32,
            };
            node.utf8_text(document_content.as_bytes())
                .ok()
                .map(|text| Element::Link {
                    link: text.to_string(),
                    start,
                    end,
                })
        }
        _ => None,
    }
}

pub(crate) fn retrieve_keyword_at_position(
    doc: &mut Document,
    parser: &mut tree_sitter::Parser,
    cursor_line: usize,
    cursor_character: usize,
) -> Option<String> {
    let document_content = doc.full.get_content(None);
    doc.tree = parser.parse(document_content, doc.tree.as_ref());
    let tree = doc.tree.as_ref()?;

    let mut query_cursor = tree_sitter::QueryCursor::new();
    let document_bytes = document_content.as_bytes();

    static KEYWORD_QUERY: LazyLock<tree_sitter::Query> = LazyLock::new(|| {
        tree_sitter::Query::new(
            &tree_sitter_mdn::LANGUAGE.into(),
            r#"
            [ (ident)
              ("{{")
              ("}}")
            ] @keywords
            "#,
        )
        .expect("Failed to create keyword query")
    });

    find_keyword_at_position(
        &mut query_cursor,
        &KEYWORD_QUERY,
        tree.root_node(),
        document_bytes,
        cursor_line,
        cursor_character,
    )
}

fn find_keyword_at_position(
    query_cursor: &mut tree_sitter::QueryCursor,
    query: &tree_sitter::Query,
    root_node: tree_sitter::Node,
    document_bytes: &[u8],
    cursor_line: usize,
    cursor_character: usize,
) -> Option<String> {
    let iter = query_cursor.matches(query, root_node, document_bytes);
    for match_ in iter {
        for capture in match_.captures {
            let node = capture.node;
            let start_position = node.start_position();
            let end_position = node.end_position();

            if is_within_cursor_range(start_position, end_position, cursor_line, cursor_character) {
                return node.utf8_text(document_bytes).ok().map(String::from);
            }
        }
    }
    None
}

fn is_within_cursor_range(
    start_position: tree_sitter::Point,
    end_position: tree_sitter::Point,
    cursor_line: usize,
    cursor_character: usize,
) -> bool {
    start_position.row == cursor_line
        && end_position.row == cursor_line
        && start_position.column <= cursor_character
        && end_position.column >= cursor_character
}
