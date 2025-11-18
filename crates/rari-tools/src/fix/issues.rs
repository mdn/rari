use std::fs::File;
use std::io::{BufWriter, Write};

use rari_doc::issues::{DIssue, IN_MEMORY};
use rari_doc::pages::page::{Page, PageBuilder, PageLike};
use tracing::{Level, span};

use crate::error::ToolError;

#[derive(Default, Debug, Clone, Copy)]
struct OLCMapper {
    offset: usize,
    line: usize,
    column: usize,
}

pub fn get_fixable_issues(page: &Page) -> Result<Vec<DIssue>, ToolError> {
    let file = page.full_path().to_string_lossy();
    let span = span!(
        Level::ERROR,
        "page",
        locale = page.locale().as_url_str(),
        slug = page.slug(),
        file = file.as_ref()
    );
    let enter = span.enter();
    let _ = page.build()?;
    drop(enter);
    let mut issues = {
        let m = IN_MEMORY.get_events();
        let (_, req_issues) = m
            .remove(page.full_path().to_string_lossy().as_ref())
            .unwrap_or_default();
        req_issues
            .into_iter()
            .filter_map(|issue| DIssue::from_issue(issue, page))
            .filter_map(|dissue| {
                let display_issue = dissue.display_issue();
                if display_issue.suggestion.is_some()
                    && display_issue.fixable.unwrap_or_default()
                    && display_issue.column.is_some()
                    && display_issue.line.is_some()
                {
                    Some(dissue)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    issues.sort_by(|a, b| {
        if a.display_issue().line == b.display_issue().line {
            a.display_issue().column.cmp(&b.display_issue().column)
        } else {
            a.display_issue().line.cmp(&b.display_issue().line)
        }
    });

    Ok(issues)
}

pub fn actual_offset(raw: &str, dissue: &DIssue) -> usize {
    let olc = OLCMapper::default();
    let new_line = dissue.display_issue().line.unwrap_or_default() as usize - 1;
    let new_column = dissue.display_issue().column.unwrap_or_default() as usize - 1;
    if let Some(offset) = calc_offset(raw, olc, new_line, new_column) {
        if let DIssue::BrokenLink {
            display_issue: _,
            href: Some(href),
        } = dissue
            && let Some(start) = raw[offset..].find(href)
        {
            let href_offset = offset + start;

            let actual_offset = href_offset + href.len();

            return actual_offset;
        }
        return offset;
    }

    0
}

#[derive(Debug, Clone)]
pub struct SearchReplaceWithOffset {
    offset: usize,
    search: String,
    replace: String,
}

pub fn collect_suggestions(raw: &str, issues: &[DIssue]) -> Vec<SearchReplaceWithOffset> {
    let mut suggestions = issues
        .iter()
        .filter_map(|dissue| {
            let offset = actual_offset(raw, dissue);
            if let DIssue::BrokenLink {
                display_issue,
                href: Some(href),
            } = dissue
                && let Some(suggestion) = display_issue.suggestion.as_deref()
            {
                Some(SearchReplaceWithOffset {
                    offset: offset - href.len(),
                    search: href.into(),
                    replace: suggestion.into(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    suggestions.sort_by(|a, b| a.offset.cmp(&b.offset));

    suggestions
}

pub fn fix_page(page: &Page) -> Result<bool, ToolError> {
    let issues = get_fixable_issues(page)?;

    let raw = page.raw_content();

    let suggestions = collect_suggestions(raw, &issues);

    let fixed = apply_suggestions(raw, &suggestions)?;
    let is_fixed = fixed != raw;
    if is_fixed {
        tracing::info!("updating {}", page.full_path().display());
        let file = File::create(page.full_path()).unwrap();
        let mut buffed = BufWriter::new(file);
        buffed.write_all(fixed.as_bytes())?;
    }
    Ok(is_fixed)
}

pub fn apply_suggestions(
    raw: &str,
    suggestions: &[SearchReplaceWithOffset],
) -> Result<String, ToolError> {
    let mut result = Vec::new();
    let mut current_offset = 0;

    for suggestion in suggestions {
        // Skip this suggestion if it overlaps with previously applied region
        if suggestion.offset < current_offset {
            tracing::debug!(
                "Skipping overlapping suggestion at offset {} (current offset: {})",
                suggestion.offset,
                current_offset
            );
            continue;
        }

        // Add the unchanged portion before this suggestion
        if suggestion.offset > current_offset {
            result.push(&raw[current_offset..suggestion.offset]);
        }

        // Add the suggestion
        result.push(&suggestion.replace);

        // Update current offset to the end of the replaced region
        current_offset = suggestion.offset + suggestion.search.len();
    }

    // Add any remaining content after the last suggestion
    if current_offset < raw.len() {
        result.push(&raw[current_offset..]);
    }

    Ok(result.join(""))
}

fn calc_offset(input: &str, olc: OLCMapper, new_line: usize, new_column: usize) -> Option<usize> {
    let OLCMapper {
        offset,
        line,
        column,
    } = olc;
    let lines = new_line - line;

    let offset = if new_line > line {
        let begin_of_line = input[offset..]
            .lines()
            .take(lines)
            .map(|line| line.len() + 1)
            .sum::<usize>();
        let new_column_offset = new_column;
        Some(offset + begin_of_line + new_column_offset)
    } else if new_line == line && new_column > column {
        Some(offset + (new_column - column))
    } else {
        tracing::warn!("skipping issues");
        None
    };
    if let Some(offset) = offset {
        let mut index = offset;
        while !input.is_char_boundary(index) {
            index -= 1;
        }
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use rari_doc::issues::{DisplayIssue, IssueType};

    #[test]
    fn test_apply_suggestions_with_duplicate_offsets() {
        let raw = "[Box Alignment][box-alignment]\n\
                   [Box Alignment][box-alignment]\n\
                   \n\
                   [box-alignment]: /en-US/docs/Web/CSS/CSS_box_alignment\n";

        let suggestions = vec![
            SearchReplaceWithOffset {
                offset: 80,
                search: "/en-US/docs/Web/CSS/CSS_box_alignment".to_string(),
                replace: "/en-US/docs/Web/CSS/Guides/Box_alignment".to_string(),
            },
            SearchReplaceWithOffset {
                offset: 80,
                search: "/en-US/docs/Web/CSS/CSS_box_alignment".to_string(),
                replace: "/en-US/docs/Web/CSS/Guides/Box_alignment".to_string(),
            },
        ];

        let result = apply_suggestions(raw, &suggestions).unwrap();

        let expected = "[Box Alignment][box-alignment]\n\
                        [Box Alignment][box-alignment]\n\
                        \n\
                        [box-alignment]: /en-US/docs/Web/CSS/Guides/Box_alignment\n";

        assert_eq!(result, expected);
    }

    #[test]
    fn test_collect_suggestions_with_duplicate_broken_links() {
        let raw = "---\n\
title: CSS layout cookbook\n\
short-title: Layout cookbook\n\
slug: Web/CSS/How_to/Layout_cookbook\n\
page-type: landing-page\n\
sidebar: cssref\n\
---\n\
[Box Alignment][box-alignment]\n\
[Flexbox][flexbox] [Box Alignment][box-alignment]\n\
\n\
[flexbox]: /en-US/docs/Web/CSS/CSS_flexible_box_layout\n\
[box-alignment]: /en-US/docs/Web/CSS/CSS_box_alignment\n";

        let issues = vec![
            DIssue::BrokenLink {
                display_issue: DisplayIssue {
                    id: 1,
                    explanation: Some("/en-US/docs/Web/CSS/CSS_box_alignment is a redirect".to_string()),
                    suggestion: Some("/en-US/docs/Web/CSS/Guides/Box_alignment".to_string()),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(9),
                    column: Some(1),
                    end_line: Some(9),
                    end_column: Some(30),
                    source_context: Some("\n[Box Alignment][box-alignment]\n^\n[Flexbox][flexbox] [Box Alignment][box-alignment]\n\n[flexbox]: /en-US/docs/Web/CSS/CSS_flexible_box_layout\n".to_string()),
                    filepath: Some("/path/to/layout_cookbook/index.md".to_string()),
                    name: IssueType::RedirectedLink,
                },
                href: Some("/en-US/docs/Web/CSS/CSS_box_alignment".to_string()),
            },
            DIssue::BrokenLink {
                display_issue: DisplayIssue {
                    id: 2,
                    explanation: Some("/en-US/docs/Web/CSS/CSS_flexible_box_layout is a redirect".to_string()),
                    suggestion: Some("/en-US/docs/Web/CSS/Guides/Flexible_box_layout".to_string()),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(10),
                    column: Some(1),
                    end_line: Some(10),
                    end_column: Some(30),
                    source_context: Some("\n[Box Alignment][box-alignment]\n[Flexbox][flexbox] [Box Alignment][box-alignment]\n^\n\n[flexbox]: /en-US/docs/Web/CSS/CSS_flexible_box_layout\n[box-alignment]: /en-US/docs/Web/CSS/CSS_box_alignment\n".to_string()),
                    filepath: Some("/path/to/layout_cookbook/index.md".to_string()),
                    name: IssueType::RedirectedLink,
                },
                href: Some("/en-US/docs/Web/CSS/CSS_flexible_box_layout".to_string()),
            },
            DIssue::BrokenLink {
                display_issue: DisplayIssue {
                    id: 3,
                    explanation: Some("/en-US/docs/Web/CSS/CSS_box_alignment is a redirect".to_string()),
                    suggestion: Some("/en-US/docs/Web/CSS/Guides/Box_alignment".to_string()),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(10),
                    column: Some(20),
                    end_line: Some(10),
                    end_column: Some(49),
                    source_context: Some("\n[Box Alignment][box-alignment]\n[Flexbox][flexbox] [Box Alignment][box-alignment]\n-------------------^\n\n[flexbox]: /en-US/docs/Web/CSS/CSS_flexible_box_layout\n[box-alignment]: /en-US/docs/Web/CSS/CSS_box_alignment\n".to_string()),
                    filepath: Some("/path/to/layout_cookbook/index.md".to_string()),
                    name: IssueType::RedirectedLink,
                },
                href: Some("/en-US/docs/Web/CSS/CSS_box_alignment".to_string()),
            },
        ];

        let suggestions = collect_suggestions(raw, &issues);

        // Both issues should produce suggestions with the same offset (80)
        // since they both reference the same link definition on line 4
        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0].offset, 234);
        assert_eq!(
            suggestions[0].search,
            "/en-US/docs/Web/CSS/CSS_flexible_box_layout"
        );
        assert_eq!(
            suggestions[0].replace,
            "/en-US/docs/Web/CSS/Guides/Flexible_box_layout"
        );
        assert_eq!(suggestions[1].offset, 295);
        assert_eq!(
            suggestions[1].search,
            "/en-US/docs/Web/CSS/CSS_box_alignment"
        );
        assert_eq!(
            suggestions[1].replace,
            "/en-US/docs/Web/CSS/Guides/Box_alignment"
        );
        assert_eq!(suggestions[1].offset, 295);
        assert_eq!(
            suggestions[1].search,
            "/en-US/docs/Web/CSS/CSS_box_alignment"
        );
        assert_eq!(
            suggestions[1].replace,
            "/en-US/docs/Web/CSS/Guides/Box_alignment"
        );
    }
}
