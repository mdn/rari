use std::fs::File;
use std::io::{BufWriter, Write};

use rari_doc::issues::{DIssue, IN_MEMORY};
use rari_doc::pages::page::{Page, PageBuilder, PageLike};
use tracing::{Level, span};

use crate::error::ToolError;

/// Offset-Line-Column mapper for tracking position during offset calculations.
///
/// All values use byte-based measurements for internal processing.
#[derive(Default, Debug, Clone, Copy)]
struct OLCMapper {
    /// Byte offset from start of content
    offset: usize,
    /// Line number (0-based)
    line: usize,
    /// Column in BYTES from start of line (0-based)
    column: usize,
}

pub fn get_fixable_issues(page: &Page) -> Result<Vec<DIssue>, ToolError> {
    let _ = page.build()?;

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

#[derive(Debug, PartialEq, Eq)]
pub struct SearchReplaceWithOffset {
    /// Byte offset in the source where the search string begins
    offset: usize,
    /// Text to find at the specified offset
    search: String,
    /// Text to replace the search string with
    replace: String,
}

/// Converts issues into offset-based search/replace suggestions
pub fn collect_suggestions(raw: &str, issues: &[DIssue]) -> Vec<SearchReplaceWithOffset> {
    let mut suggestions = issues
        .iter()
        .filter_map(|dissue| {
            let offset_end = actual_offset(raw, dissue);
            if let DIssue::BrokenLink {
                display_issue,
                href: Some(href),
            } = dissue
                && let Some(suggestion) = display_issue.suggestion.as_deref()
            {
                // actual_offset returns the END of the href
                // We need to find the START by searching backward for the href
                // Use the byte length as an estimate, but verify by searching
                let mut search_start = offset_end.saturating_sub(href.len() + 10); // Add margin for safety

                // Ensure search_start is on a char boundary
                while search_start > 0 && !raw.is_char_boundary(search_start) {
                    search_start -= 1;
                }

                // Ensure offset_end is on a char boundary
                let mut offset_end_adjusted = offset_end;
                while offset_end_adjusted < raw.len() && !raw.is_char_boundary(offset_end_adjusted)
                {
                    offset_end_adjusted += 1;
                }

                if let Some(relative_pos) = raw[search_start..offset_end_adjusted].rfind(href) {
                    let href_start = search_start + relative_pos;

                    // Verify this is the correct match (approximately - offset_end might have been adjusted)
                    let href_end = href_start + href.len();
                    if (href_end == offset_end || href_end == offset_end_adjusted)
                        && &raw[href_start..href_end] == href
                    {
                        Some(SearchReplaceWithOffset {
                            offset: href_start,
                            search: href.into(),
                            replace: suggestion.into(),
                        })
                    } else {
                        tracing::warn!(
                            "Could not find href '{}' at expected position (end: {}, adjusted: {})",
                            href,
                            offset_end,
                            offset_end_adjusted
                        );
                        None
                    }
                } else {
                    tracing::warn!(
                        "Could not locate href '{}' before offset {}",
                        href,
                        offset_end
                    );
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    suggestions.sort_by(|a, b| a.offset.cmp(&b.offset));
    suggestions.dedup();

    suggestions
}

/// Applies search/replace suggestions to raw content, returning the modified text
pub fn apply_suggestions(
    raw: &str,
    suggestions: &[SearchReplaceWithOffset],
) -> Result<String, ToolError> {
    let mut result = Vec::new();
    let mut current_offset = 0;

    for suggestion in suggestions {
        // Ensure suggestion.offset is on a character boundary
        let suggestion_offset = if raw.is_char_boundary(suggestion.offset) {
            suggestion.offset
        } else {
            // Adjust to the nearest valid character boundary (previous char start)
            let mut offset = suggestion.offset;
            while offset > 0 && !raw.is_char_boundary(offset) {
                offset -= 1;
            }
            tracing::warn!(
                "Adjusted suggestion offset from {} to {} (not on char boundary)",
                suggestion.offset,
                offset
            );
            offset
        };

        // Skip this suggestion if it overlaps with previously applied region
        if suggestion_offset < current_offset {
            tracing::warn!(
                "Cannot apply suggestion ('{}' -> '{}'), because it overlaps with another suggestion.",
                suggestion.search,
                suggestion.replace
            );
            continue;
        }

        // Add the unchanged portion before this suggestion
        if suggestion_offset > current_offset {
            result.push(&raw[current_offset..suggestion_offset]);
        }

        // Validate that the search string matches what's actually in the raw content
        let end_offset = suggestion_offset + suggestion.search.len();
        if end_offset > raw.len() {
            tracing::warn!(
                "Cannot apply suggestion ('{}' -> '{}'), because its offset ({}-{}) exceeds raw content length {}",
                suggestion.search,
                suggestion.replace,
                suggestion_offset,
                end_offset,
                raw.len()
            );
            continue;
        }

        // Ensure end_offset is on a character boundary
        if !raw.is_char_boundary(end_offset) {
            tracing::warn!(
                "Cannot apply suggestion ('{}' -> '{}'), because end_offset {} is not on a char boundary",
                suggestion.search,
                suggestion.replace,
                end_offset
            );
            continue;
        }

        let actual_content = &raw[suggestion_offset..end_offset];
        if actual_content != suggestion.search {
            tracing::warn!(
                "Cannot apply suggestion ('{}' -> '{}'), because actual content at offset {} is '{}'",
                suggestion.search,
                suggestion.replace,
                suggestion_offset,
                actual_content
            );
            continue;
        }

        // Add the suggestion
        result.push(&suggestion.replace);

        // Update current offset to the end of the replaced region
        current_offset = end_offset;
    }

    // Add any remaining content after the last suggestion
    if current_offset < raw.len() {
        result.push(&raw[current_offset..]);
    }

    Ok(result.join(""))
}

pub fn fix_page(page: &Page) -> Result<bool, ToolError> {
    let span = span!(
        Level::ERROR,
        "page",
        locale = page.locale().as_url_str(),
        slug = page.slug(),
        file = page.full_path().to_string_lossy().as_ref()
    );
    let enter = span.enter();

    let issues = get_fixable_issues(page)?;

    let raw = page.raw_content();

    let suggestions = collect_suggestions(raw, &issues);

    let fixed = apply_suggestions(raw, &suggestions)?;
    drop(enter);
    let is_fixed = fixed != raw;
    if is_fixed {
        tracing::info!("updating {}", page.full_path().display());
        let file = File::create(page.full_path()).unwrap();
        let mut buffed = BufWriter::new(file);
        buffed.write_all(fixed.as_bytes())?;
    }
    Ok(is_fixed)
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
    // Verify the calculated offset is on a UTF-8 character boundary
    if let Some(mut offset_value) = offset {
        if offset_value < input.len() && !input.is_char_boundary(offset_value) {
            tracing::warn!(
                "calculated offset {} is not on char boundary - adjusting (this may indicate a bug)",
                offset_value
            );
            // Move backwards to the nearest char boundary
            while offset_value > 0 && !input.is_char_boundary(offset_value) {
                offset_value -= 1;
            }
            return Some(offset_value);
        }
    }
    offset
}

pub fn actual_offset(raw: &str, dissue: &DIssue) -> usize {
    let olc = OLCMapper::default();
    let new_line = dissue.display_issue().line.unwrap_or_default() as usize - 1;
    // DisplayIssue.column is in CHARACTERS (1-based), need to convert to BYTES (0-based) for calc_offset
    let char_column = dissue.display_issue().column.unwrap_or_default() as usize - 1;

    // Convert character column to byte column
    let new_column = if let Some(line_content) = raw.lines().nth(new_line) {
        use rari_doc::position_utils::char_to_byte_column;
        char_to_byte_column(line_content, char_column)
    } else {
        char_column // Fallback: use as-is if line not found
    };
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
        assert_eq!(suggestions.len(), 2);
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
    }

    #[test]
    fn test_fix_link_with_multibyte_chars() {
        // Test that link fixing works correctly with multi-byte UTF-8 characters
        // "Café" has é which is 2 bytes, and "🔥" is 4 bytes
        let raw = "---\n\
title: Test\n\
---\n\
Café 🔥 [Link](/en-US/docs/old)\n";

        // The link starts at:
        // Line 3 (0-indexed): "Café 🔥 [Link](/en-US/docs/old)"
        // "Café " = 4 chars, 6 bytes (C=1, a=1, f=1, é=2, space=1)
        // "🔥 " = 2 chars, 5 bytes (emoji=4, space=1)
        // "[Link]" starts at char 6, byte 11

        // Create an issue with CHARACTER positions (as DisplayIssue now uses)
        let issues = vec![DIssue::BrokenLink {
            display_issue: DisplayIssue {
                id: 1,
                explanation: Some("/en-US/docs/old is a redirect".to_string()),
                suggestion: Some("/en-US/docs/new".to_string()),
                fixable: Some(true),
                fixed: false,
                line: Some(4),    // 1-based line number
                column: Some(13), // 1-based CHARACTER position of '/' in the URL
                end_line: Some(4),
                end_column: Some(29), // End of URL in characters
                source_context: None,
                filepath: Some("/path/to/test.md".to_string()),
                name: IssueType::RedirectedLink,
            },
            href: Some("/en-US/docs/old".to_string()),
        }];

        let suggestions = collect_suggestions(raw, &issues);

        assert_eq!(suggestions.len(), 1);
        // The byte offset should be calculated correctly despite multi-byte chars
        // Line 0: "---" = 3 + 1 newline = 4
        // Line 1: "title: Test" = 11 + 1 newline = 12
        // Line 2: "---" = 3 + 1 newline = 4
        // Line 3: "Café 🔥 [Link](" = 18 bytes (Café=5, space=1, 🔥=4, space=1, [Link](=7)
        // Total: 4 + 12 + 4 + 18 = 38 bytes
        let expected_offset = 38; // Start of "/en-US/docs/old" in bytes
        assert_eq!(
            suggestions[0].offset, expected_offset,
            "Offset should account for multi-byte characters"
        );
        assert_eq!(suggestions[0].search, "/en-US/docs/old");
        assert_eq!(suggestions[0].replace, "/en-US/docs/new");
    }

    #[test]
    fn test_apply_suggestions_with_invalid_char_boundary() {
        // Test that apply_suggestions handles offsets that aren't on character boundaries
        // This can happen when offset calculation results in a byte offset in the middle of a multi-byte char
        let raw = "Café [link](url)";
        // "Café" = C(1) + a(1) + f(1) + é(2 bytes at positions 3-4) = 5 bytes total
        // If we mistakenly calculate offset as 4, that's in the middle of 'é'

        let suggestions = vec![SearchReplaceWithOffset {
            offset: 4, // This is NOT on a char boundary (inside 'é' which spans bytes 3-4)
            search: " ".to_string(), // The space after "Café"
            replace: "_".to_string(),
        }];

        // This should not panic, but should adjust the offset to a valid boundary
        let result = apply_suggestions(raw, &suggestions);

        // The function should handle the invalid offset gracefully
        // It will adjust to byte 3 (start of 'é') and fail to find the search string there
        // So the suggestion won't be applied, but it shouldn't panic
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_suggestions_multibyte_replacement() {
        // Test a complete flow with multi-byte characters
        let raw = "été [link](old)";
        // "été " = é(2) + t(1) + é(2) + space(1) = 6 bytes
        // "[link]" = 6 bytes, "(" = 1 byte
        // So "old" starts at byte 13

        let suggestions = vec![SearchReplaceWithOffset {
            offset: 13,
            search: "old".to_string(),
            replace: "new".to_string(),
        }];

        let result = apply_suggestions(raw, &suggestions).unwrap();
        assert_eq!(result, "été [link](new)");
    }
}
