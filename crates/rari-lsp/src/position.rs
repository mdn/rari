use std::sync::LazyLock;

use pulldown_cmark::{Event, Parser, Tag};
use streaming_iterator::StreamingIterator;

fn row_col_to_offset(text: &str, row: usize, col: usize) -> Option<usize> {
    text.lines()
        .take(row)
        .map(|line| line.len() + 1)
        .sum::<usize>()
        .checked_add(col)
        .filter(|&offset| offset <= text.len())
}

#[derive(Clone, Debug)]
pub enum Element {
    Link { link: String },
}

pub(crate) fn retrieve_element_at_position(
    document_content: &str,
    cursor_line: usize,
    cursor_character: usize,
) -> Option<Element> {
    if let Some(offset) = row_col_to_offset(document_content, cursor_line, cursor_character) {
        let mut context = vec![];

        if let Some((event, _)) = Parser::new(document_content)
            .into_offset_iter()
            .take_while(|(event, range)| {
                (!matches!(event, Event::End(_)) && range.contains(&offset)) || range.end < offset
            })
            .filter_map(|(event, range)| {
                println!("{event:?} {offset} {range:?}");
                if range.contains(&offset) {
                    match event {
                        Event::Start(_) => {
                            context.push((event.clone(), range.clone()));
                            return Some((event, range));
                        }
                        Event::End(_) => {
                            context.pop();
                        }
                        _ => return Some((event, range)),
                    }
                }
                None
            })
            .last()
        {
            match event {
                Event::Text(s) => {
                    if let Some((Event::Start(Tag::Link { dest_url, .. }), _)) = context.last() {
                        return Some(Element::Link {
                            link: dest_url.to_string(),
                        });
                    }
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    return Some(Element::Link {
                        link: dest_url.to_string(),
                    });
                }
                _ => (),
            }
        }
    }
    None
}
pub(crate) fn retrieve_keyword_at_position(
    document_content: &str,
    parser: &mut tree_sitter::Parser,
    syntax_tree: &mut Option<tree_sitter::Tree>,
    cursor_line: usize,
    cursor_character: usize,
) -> Option<String> {
    *syntax_tree = parser.parse(document_content, syntax_tree.as_ref());
    let tree = syntax_tree.as_ref()?;

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
    let mut iter = query_cursor.matches(query, root_node, document_bytes);
    while let Some(match_) = iter.next() {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_1() {
        let md = r#"#foo

bar
2000
[](foo)

`dara`
"#;
        let e = retrieve_element_at_position(md, 4, 4);
        println!("{e:?}");
    }
}
