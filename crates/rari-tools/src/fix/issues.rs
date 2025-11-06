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

pub fn fix_page(page: &Page) -> Result<bool, ToolError> {
    let issues = get_fixable_issues(page)?;

    let raw = page.raw_content();
    let fixed = fix_issues(raw, &issues)?;
    let is_fixed = fixed != raw;
    if is_fixed {
        tracing::info!("updating {}", page.full_path().display());
        let file = File::create(page.full_path()).unwrap();
        let mut buffed = BufWriter::new(file);
        buffed.write_all(fixed.as_bytes())?;
    }
    Ok(is_fixed)
}

pub fn fix_issues(raw: &str, issues: &[DIssue]) -> Result<String, ToolError> {
    let (olc, mut fixed) =
        issues
            .iter()
            .fold((OLCMapper::default(), vec![]), |(olc, mut acc), dissue| {
                let olc = fix_issue(&mut acc, dissue, raw, olc);
                (olc, acc)
            });
    fixed.push(&raw[olc.offset..]);
    Ok(fixed.join(""))
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

fn fix_issue<'a>(
    acc: &mut Vec<&'a str>,
    dissue: &'a DIssue,
    raw: &'a str,
    olc: OLCMapper,
) -> OLCMapper {
    let new_line = dissue.display_issue().line.unwrap_or_default() as usize - 1;
    let new_column = dissue.display_issue().column.unwrap_or_default() as usize - 1;
    if let Some(offset) = calc_offset(raw, olc, new_line, new_column) {
        #[allow(clippy::single_match)]
        match dissue {
            DIssue::BrokenLink {
                display_issue,
                href: Some(href),
            } => {
                if let Some(start) = raw[offset..].find(href) {
                    let href_offset = offset + start;

                    if href_offset > olc.offset {
                        acc.push(&raw[olc.offset..href_offset]);
                    }
                    let fix = display_issue.suggestion.as_deref().unwrap_or_default();
                    let new_offset = href_offset + href.len();
                    acc.push(fix);
                    return OLCMapper {
                        offset: new_offset,
                        line: new_line,
                        column: new_column + start + href.len(),
                    };
                }
            }
            _ => {}
        }
    }
    olc
}
