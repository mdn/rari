use std::fs::File;
use std::io::{BufWriter, Write};

use rari_doc::issues::{DIssue, IN_MEMORY};
use rari_doc::pages::page::{Page, PageBuilder, PageLike};
use rari_doc::position_utils::{Direction, adjust_to_char_boundary, calculate_line_start_offset};
use tracing::{Level, span};

use crate::error::ToolError;

/// Extracts the slug from an href by stripping the leading slash and /docs/ prefix.
/// Returns the slug portion, or the original stripped string if no /docs/ is found.
///
/// Examples:
/// - "/en-US/docs/Web/API/Foo" → "Web/API/Foo"
/// - "/Web/API/Foo" → "Web/API/Foo"
fn extract_slug_from_href(href: &str) -> &str {
    if let Some(stripped) = href.strip_prefix("/") {
        stripped
            .split_once("/docs/")
            .map(|(_, rest)| rest)
            .unwrap_or(stripped)
    } else {
        href
    }
}

/// Searches for an href in text, trying the full href first, then falling back to just the slug.
///
/// This handles template-generated links where the markdown contains only the slug portion
/// (e.g., "Web/API/Foo") rather than the full href ("/en-US/docs/Web/API/Foo").
///
/// # Arguments
/// * `href` - The full href to search for
/// * `search_fn` - Function that searches for text and returns an offset if found
///
/// # Returns
/// Tuple of (found_offset, text_that_was_found) if successful, None otherwise
fn search_with_slug_fallback<F>(href: &str, mut search_fn: F) -> Option<(usize, String)>
where
    F: FnMut(&str) -> Option<usize>,
{
    // Try full href first
    if let Some(offset) = search_fn(href) {
        return Some((offset, href.to_string()));
    }

    // Fallback: try just the slug
    if href.starts_with("/") {
        let slug = extract_slug_from_href(href);
        if let Some(offset) = search_fn(slug) {
            return Some((offset, slug.to_string()));
        }
    }

    None
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
                    && display_issue.line.is_some()
                {
                    // Column is optional - if missing, we'll search from line start
                    Some(dissue)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    issues.sort_by(|a, b| {
        if a.display_issue().line == b.display_issue().line {
            // Treat None as Some(1) for consistent ordering (columns are 1-based)
            let col_a = a.display_issue().column.or(Some(1));
            let col_b = b.display_issue().column.or(Some(1));
            col_a.cmp(&col_b)
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
            let (display_issue, href) = match dissue {
                DIssue::BrokenLink {
                    display_issue,
                    href: Some(href),
                } => (display_issue, href),
                DIssue::Macros {
                    display_issue,
                    href: Some(href),
                    ..
                } => (display_issue, href),
                _ => return None,
            };
            if let Some(suggestion) = display_issue.suggestion.as_deref() {
                // The href and suggestion from HTML may contain HTML entities (&#x27; for ', &lt; for <, etc.)
                // Decode them to match the raw markdown content
                let decoded_href = html_escape::decode_html_entities(href);
                let decoded_suggestion = html_escape::decode_html_entities(suggestion);

                // Try to find the href in the markdown. First try the full href, then fallback to slug.
                // This handles template-generated links where the markdown contains only the slug.

                // actual_offset returns the END of the href
                // We need to find the START by searching backward for the text

                // Ensure offset_end is on a char boundary
                let offset_end_adjusted =
                    adjust_to_char_boundary(raw, offset_end, Direction::Forward);

                // Try 1: Search for the full href
                let try_search = |search_text: &str| -> Option<usize> {
                    let search_start = offset_end.saturating_sub(search_text.len());

                    // Ensure search_start is on a char boundary
                    let search_start =
                        adjust_to_char_boundary(raw, search_start, Direction::Backward);

                    if let Some(relative_pos) =
                        raw[search_start..offset_end_adjusted].rfind(search_text)
                    {
                        let href_start = search_start + relative_pos;
                        let href_end = href_start + search_text.len();

                        // Verify this is the correct match
                        if (href_end == offset_end || href_end == offset_end_adjusted)
                            && &raw[href_start..href_end] == search_text
                        {
                            return Some(href_start);
                        }
                    }
                    None
                };

                // Try finding the full href first, fallback to slug
                let result = search_with_slug_fallback(&decoded_href, try_search).map(
                    |(href_start, search_text)| {
                        // If we found the full href, use full suggestion; if slug, extract slug from suggestion
                        let replace_text = if search_text == decoded_href.as_ref() {
                            decoded_suggestion.to_string()
                        } else {
                            extract_slug_from_href(&decoded_suggestion).to_string()
                        };
                        (href_start, search_text, replace_text)
                    },
                );

                if let Some((href_start, search_text, replace_text)) = result {
                    Some(SearchReplaceWithOffset {
                        offset: href_start,
                        search: search_text,
                        replace: replace_text,
                    })
                } else {
                    // Show context around the offset for debugging
                    let search_start = offset_end.saturating_sub(decoded_href.len());
                    let context_start = search_start;
                    let context_end = offset_end_adjusted.min(raw.len());
                    let context = &raw[context_start..context_end];
                    tracing::warn!(
                        "Could not locate '{}' before offset {} (searched region: {:?})",
                        decoded_href,
                        offset_end,
                        context
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
        let suggestion_offset =
            adjust_to_char_boundary(raw, suggestion.offset, Direction::Backward);
        if suggestion_offset != suggestion.offset {
            tracing::warn!(
                "Adjusted suggestion offset from {} to {} (not on char boundary)",
                suggestion.offset,
                suggestion_offset
            );
        }

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

pub fn actual_offset(raw: &str, dissue: &DIssue) -> usize {
    let href = match dissue {
        DIssue::BrokenLink {
            href: Some(href), ..
        } => href,
        DIssue::Macros {
            href: Some(href), ..
        } => href,
        _ => return 0,
    };

    // Try to find the href in the markdown. First try the full href, then fallback to slug.
    let decoded_href = html_escape::decode_html_entities(href);

    // Get line information from the issue
    // Note: Column information refers to rendered HTML positions, not markdown source positions,
    // so we search from the line start instead. Column is still useful for ordering issues.
    let line_num = dissue.display_issue().line;
    if let Some(line) = line_num {
        let line_idx = (line as usize).saturating_sub(1);
        let line_start_offset = calculate_line_start_offset(raw, line_idx);

        // Try searching for the full href first, fallback to slug
        if let Some((offset, _found_text)) =
            search_with_slug_fallback(&decoded_href, |search_text| {
                if let Some(start) = raw[line_start_offset..].find(search_text) {
                    let text_offset = line_start_offset + start;
                    Some(text_offset + search_text.len())
                } else {
                    None
                }
            })
        {
            return offset;
        }
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

    #[test]
    fn test_fix_link_with_html_entities() {
        // Test that HTML entities in hrefs are properly decoded when searching the raw markdown
        // This happens when hrefs come from HTML output with entities like &#x27; (apostrophe)
        let raw = "---\n\
title: Test\n\
---\n\
[Link](/fr/docs/Web/SVG/Attribute#attributs_d'événement)\n";

        // The href in the issue contains HTML entities (as it comes from HTML output)
        // but the raw markdown contains literal characters
        let issues = vec![DIssue::BrokenLink {
            display_issue: DisplayIssue {
                id: 1,
                explanation: Some("Redirect detected".to_string()),
                suggestion: Some("/fr/docs/Web/SVG/Attribute#evenements".to_string()),
                fixable: Some(true),
                fixed: false,
                line: Some(4),   // 1-based line number
                column: Some(8), // 1-based CHARACTER position
                end_line: Some(4),
                end_column: Some(54),
                source_context: None,
                filepath: Some("/path/to/test.md".to_string()),
                name: IssueType::RedirectedLink,
            },
            // href contains HTML entities (&#x27; for apostrophe, é is already UTF-8)
            href: Some("/fr/docs/Web/SVG/Attribute#attributs_d&#x27;événement".to_string()),
        }];

        let suggestions = collect_suggestions(raw, &issues);

        assert_eq!(
            suggestions.len(),
            1,
            "Should find the href despite HTML entities"
        );
        // Line 0: "---" = 3 + 1 newline = 4
        // Line 1: "title: Test" = 11 + 1 newline = 12
        // Line 2: "---" = 3 + 1 newline = 4
        // Line 3: "[Link](" = 7 bytes
        // Total: 4 + 12 + 4 + 7 = 27 bytes
        let expected_offset = 27; // Start of the href in bytes
        assert_eq!(suggestions[0].offset, expected_offset);
        // Both search and replace should be DECODED (with literal apostrophes)
        assert_eq!(
            suggestions[0].search,
            "/fr/docs/Web/SVG/Attribute#attributs_d'événement"
        );
        assert_eq!(
            suggestions[0].replace,
            "/fr/docs/Web/SVG/Attribute#evenements"
        );
    }

    #[test]
    fn test_fix_link_with_html_entities_in_both() {
        // Test where BOTH href and suggestion contain HTML entities
        let raw = "---\n\
title: Test\n\
---\n\
[Link](/fr/docs/Web/SVG/Attribute#attributs_d'événement)\n";

        let issues = vec![DIssue::BrokenLink {
            display_issue: DisplayIssue {
                id: 1,
                explanation: Some("Redirect detected".to_string()),
                // Suggestion ALSO has HTML entities (this can happen in real redirects)
                suggestion: Some(
                    "/fr/docs/Web/SVG/Reference/Attribute#attributs_d&#x27;événement_globaux"
                        .to_string(),
                ),
                fixable: Some(true),
                fixed: false,
                line: Some(4),
                column: Some(8),
                end_line: Some(4),
                end_column: Some(54),
                source_context: None,
                filepath: Some("/path/to/test.md".to_string()),
                name: IssueType::RedirectedLink,
            },
            href: Some("/fr/docs/Web/SVG/Attribute#attributs_d&#x27;événement".to_string()),
        }];

        let suggestions = collect_suggestions(raw, &issues);

        assert_eq!(suggestions.len(), 1);
        // Both search and replace should be decoded to literal characters
        assert_eq!(
            suggestions[0].search,
            "/fr/docs/Web/SVG/Attribute#attributs_d'événement"
        );
        assert_eq!(
            suggestions[0].replace,
            "/fr/docs/Web/SVG/Reference/Attribute#attributs_d'événement_globaux"
        );

        // Apply the suggestion and verify the result
        let result = apply_suggestions(raw, &suggestions).unwrap();
        assert_eq!(
            result,
            "---\n\
title: Test\n\
---\n\
[Link](/fr/docs/Web/SVG/Reference/Attribute#attributs_d'événement_globaux)\n"
        );
    }

    #[test]
    fn test_slug_fallback_in_template() {
        // Test that slug fallback works when templates contain slugs instead of full hrefs
        // This is common in templates like {{PreviousNext("slug1", "slug2")}}
        let raw = "---\n\
title: Indexed collections\n\
slug: Web/JavaScript/Guide/Indexed_collections\n\
---\n\
{{PreviousNext(\"Web/JavaScript/Guide/Regular_Expressions\", \"Web/JavaScript/Guide/Keyed_Collections\")}}\n\
\n\
Some content here.\n";

        // The issue contains a full href (as it comes from the rendered HTML)
        // but the raw markdown only contains the slug
        let issues = vec![
            DIssue::Macros {
                display_issue: DisplayIssue {
                    id: 1,
                    explanation: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Regular_Expressions should be /en-US/docs/Web/JavaScript/Guide/Regular_expressions"
                            .to_string(),
                    ),
                    suggestion: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Regular_expressions".to_string(),
                    ),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(5),
                    column: Some(17),
                    end_line: Some(5),
                    end_column: Some(77),
                    source_context: None,
                    filepath: Some("/path/to/indexed_collections/index.md".to_string()),
                    name: IssueType::TemplIllCasedLink,
                },
                macro_name: Some("PreviousNext".to_string()),
                href: Some("/en-US/docs/Web/JavaScript/Guide/Regular_Expressions".to_string()),
            },
            DIssue::Macros {
                display_issue: DisplayIssue {
                    id: 2,
                    explanation: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Keyed_Collections should be /en-US/docs/Web/JavaScript/Guide/Keyed_collections"
                            .to_string(),
                    ),
                    suggestion: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Keyed_collections".to_string(),
                    ),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(5),
                    column: Some(81),
                    end_line: Some(5),
                    end_column: Some(139),
                    source_context: None,
                    filepath: Some("/path/to/indexed_collections/index.md".to_string()),
                    name: IssueType::TemplIllCasedLink,
                },
                macro_name: Some("PreviousNext".to_string()),
                href: Some("/en-US/docs/Web/JavaScript/Guide/Keyed_Collections".to_string()),
            },
        ];

        let suggestions = collect_suggestions(raw, &issues);

        // Should find both slugs and suggest fixes
        assert_eq!(suggestions.len(), 2);

        // First suggestion: should fall back to slug and fix casing
        assert_eq!(
            suggestions[0].search, "Web/JavaScript/Guide/Regular_Expressions",
            "Should use slug (not full href) for search"
        );
        assert_eq!(
            suggestions[0].replace, "Web/JavaScript/Guide/Regular_expressions",
            "Should use slug (not full href) for replacement"
        );

        // Second suggestion
        assert_eq!(
            suggestions[1].search,
            "Web/JavaScript/Guide/Keyed_Collections"
        );
        assert_eq!(
            suggestions[1].replace,
            "Web/JavaScript/Guide/Keyed_collections"
        );

        // Apply the suggestions and verify the result
        let result = apply_suggestions(raw, &suggestions).unwrap();
        assert_eq!(
            result,
            "---\n\
title: Indexed collections\n\
slug: Web/JavaScript/Guide/Indexed_collections\n\
---\n\
{{PreviousNext(\"Web/JavaScript/Guide/Regular_expressions\", \"Web/JavaScript/Guide/Keyed_collections\")}}\n\
\n\
Some content here.\n"
        );
    }

    #[test]
    fn test_template_redirected_link() {
        // Test that redirected links in templates can be fixed
        // This is for TemplRedirectedLink (not just TemplIllCasedLink)
        let raw = "---\n\
title: Indexed collections\n\
slug: Web/JavaScript/Guide/Indexed_collections\n\
---\n\
{{PreviousNext(\"Web/JavaScript/Guide/Regular_Expressions/Groups_and_Ranges\", \"Web/JavaScript/Guide/Keyed_collections\")}}\n\
\n\
Some content here.\n";

        // The first parameter is a redirect (not just ill-cased)
        let issues = vec![DIssue::Macros {
            display_issue: DisplayIssue {
                id: 1,
                explanation: Some(
                    "Macro produces link /fr/docs/Web/JavaScript/Guide/Regular_Expressions/Groups_and_Ranges which is a redirect"
                        .to_string(),
                ),
                suggestion: Some(
                    "/fr/docs/Web/JavaScript/Guide/Regular_expressions/Groups_and_backreferences"
                        .to_string(),
                ),
                fixable: Some(true),
                fixed: false,
                line: Some(5),
                column: Some(1), // Only knows the line, not the specific parameter
                end_line: Some(5),
                end_column: Some(1),
                source_context: None,
                filepath: Some("/path/to/indexed_collections/index.md".to_string()),
                name: IssueType::TemplRedirectedLink,
            },
            macro_name: Some("PreviousNext".to_string()),
            href: Some("/fr/docs/Web/JavaScript/Guide/Regular_Expressions/Groups_and_Ranges".to_string()),
        }];

        let suggestions = collect_suggestions(raw, &issues);

        // Should find the slug and suggest the fix
        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].search, "Web/JavaScript/Guide/Regular_Expressions/Groups_and_Ranges",
            "Should use slug for search"
        );
        assert_eq!(
            suggestions[0].replace,
            "Web/JavaScript/Guide/Regular_expressions/Groups_and_backreferences",
            "Should use slug for replacement with the actual redirect target"
        );

        // Apply the suggestion and verify the result
        let result = apply_suggestions(raw, &suggestions).unwrap();
        assert_eq!(
            result,
            "---\n\
title: Indexed collections\n\
slug: Web/JavaScript/Guide/Indexed_collections\n\
---\n\
{{PreviousNext(\"Web/JavaScript/Guide/Regular_expressions/Groups_and_backreferences\", \"Web/JavaScript/Guide/Keyed_collections\")}}\n\
\n\
Some content here.\n"
        );
    }

    #[test]
    fn test_slug_fallback_with_no_column() {
        // Test slug fallback when column is None (template at beginning of line)
        // This is common when {{PreviousNext}} starts at column 1
        let raw = "---\n\
title: Indexed collections\n\
slug: Web/JavaScript/Guide/Indexed_collections\n\
---\n\
{{PreviousNext(\"Web/JavaScript/Guide/Regular_Expressions\", \"Web/JavaScript/Guide/Keyed_Collections\")}}\n\
\n\
Some content here.\n";

        // When the template is at the beginning of the line, column might be None
        let issues = vec![
            DIssue::Macros {
                display_issue: DisplayIssue {
                    id: 1,
                    explanation: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Regular_Expressions should be /en-US/docs/Web/JavaScript/Guide/Regular_expressions"
                            .to_string(),
                    ),
                    suggestion: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Regular_expressions".to_string(),
                    ),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(5),
                    column: None, // No column information
                    end_line: Some(5),
                    end_column: Some(60),
                    source_context: None,
                    filepath: Some("/path/to/indexed_collections/index.md".to_string()),
                    name: IssueType::TemplIllCasedLink,
                },
                macro_name: Some("PreviousNext".to_string()),
                href: Some("/en-US/docs/Web/JavaScript/Guide/Regular_Expressions".to_string()),
            },
            DIssue::Macros {
                display_issue: DisplayIssue {
                    id: 2,
                    explanation: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Keyed_Collections should be /en-US/docs/Web/JavaScript/Guide/Keyed_collections"
                            .to_string(),
                    ),
                    suggestion: Some(
                        "/en-US/docs/Web/JavaScript/Guide/Keyed_collections".to_string(),
                    ),
                    fixable: Some(true),
                    fixed: false,
                    line: Some(5),
                    column: None, // No column information
                    end_line: Some(5),
                    end_column: Some(118),
                    source_context: None,
                    filepath: Some("/path/to/indexed_collections/index.md".to_string()),
                    name: IssueType::TemplIllCasedLink,
                },
                macro_name: Some("PreviousNext".to_string()),
                href: Some("/en-US/docs/Web/JavaScript/Guide/Keyed_Collections".to_string()),
            },
        ];

        let suggestions = collect_suggestions(raw, &issues);

        // Should still find both slugs using line-based search
        assert_eq!(suggestions.len(), 2);

        // First suggestion: should fall back to slug and fix casing
        assert_eq!(
            suggestions[0].search, "Web/JavaScript/Guide/Regular_Expressions",
            "Should use slug for search even without column info"
        );
        assert_eq!(
            suggestions[0].replace, "Web/JavaScript/Guide/Regular_expressions",
            "Should use slug for replacement even without column info"
        );

        // Second suggestion
        assert_eq!(
            suggestions[1].search,
            "Web/JavaScript/Guide/Keyed_Collections"
        );
        assert_eq!(
            suggestions[1].replace,
            "Web/JavaScript/Guide/Keyed_collections"
        );

        // Apply the suggestions and verify the result
        let result = apply_suggestions(raw, &suggestions).unwrap();
        assert_eq!(
            result,
            "---\n\
title: Indexed collections\n\
slug: Web/JavaScript/Guide/Indexed_collections\n\
---\n\
{{PreviousNext(\"Web/JavaScript/Guide/Regular_expressions\", \"Web/JavaScript/Guide/Keyed_collections\")}}\n\
\n\
Some content here.\n"
        );
    }
}
