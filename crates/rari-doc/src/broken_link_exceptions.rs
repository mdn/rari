use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

use rari_types::globals::content_root;

use crate::resolve::{strip_locale_from_url, url_to_folder_path};

const FILE_NAME: &str = "_allowed-broken-links.txt";

static EXCEPTIONS: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let path = content_root().join(FILE_NAME);
    match fs::read_to_string(&path) {
        Ok(contents) => parse_exceptions(&contents),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashSet::new(),
        Err(error) => {
            tracing::warn!("Unable to read {}: {error}", path.display());
            HashSet::new()
        }
    }
});

static UNRESOLVED_REFERENCES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn is_exception(url: &str) -> bool {
    let Some(slug) = normalize_slug(url) else {
        return false;
    };
    if EXCEPTIONS.contains(&slug) {
        UNRESOLVED_REFERENCES
            .lock()
            .expect("link exception reference lock poisoned")
            .insert(slug);
        true
    } else {
        false
    }
}

pub fn begin_reference_scan() {
    UNRESOLVED_REFERENCES
        .lock()
        .expect("link exception reference lock poisoned")
        .clear();
}

pub fn prune_unneeded(contents: &str, still_unresolved: &HashSet<String>) -> (String, usize) {
    struct Section {
        contents: String,
        separator: String,
        entries: Vec<String>,
    }

    let mut leading_blanks = String::new();
    let mut sections: Vec<Section> = Vec::new();
    let mut current = Section {
        contents: String::new(),
        separator: String::new(),
        entries: Vec::new(),
    };
    for line in contents.split_inclusive('\n') {
        if line.trim().is_empty() {
            if current.contents.is_empty() {
                if let Some(previous) = sections.last_mut() {
                    previous.separator.push_str(line);
                } else {
                    leading_blanks.push_str(line);
                }
            } else {
                current.separator.push_str(line);
                sections.push(current);
                current = Section {
                    contents: String::new(),
                    separator: String::new(),
                    entries: Vec::new(),
                };
            }
        } else {
            if let Some(entry) = parse_entry(line) {
                current.entries.push(entry);
            }
            current.contents.push_str(line);
        }
    }
    if !current.contents.is_empty() {
        sections.push(current);
    }

    let mut kept = Vec::new();
    let mut removed = 0;
    for (index, section) in sections.iter().enumerate() {
        let surviving_entries = section
            .entries
            .iter()
            .filter(|slug| still_unresolved.contains(*slug))
            .count();
        removed += section.entries.len() - surviving_entries;
        if section.entries.is_empty() || surviving_entries > 0 {
            kept.push(index);
        }
    }

    let mut output = leading_blanks;
    for (kept_position, index) in kept.iter().copied().enumerate() {
        if let Some(previous_index) = kept_position.checked_sub(1).map(|i| kept[i]) {
            if index == previous_index + 1 {
                output.push_str(&sections[previous_index].separator);
            } else {
                output.push('\n');
            }
        }
        for line in sections[index].contents.split_inclusive('\n') {
            if parse_entry(line).is_none_or(|slug| still_unresolved.contains(&slug)) {
                output.push_str(line);
            }
        }
    }
    if let Some(last_index) = kept.last()
        && *last_index == sections.len() - 1
    {
        output.push_str(&sections[*last_index].separator);
    }

    (output, removed)
}

pub fn prune_file() -> std::io::Result<usize> {
    let path = content_root().join(FILE_NAME);
    prune_file_at(&path)
}

fn prune_file_at(path: &Path) -> std::io::Result<usize> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    let referenced = UNRESOLVED_REFERENCES
        .lock()
        .expect("link exception reference lock poisoned")
        .clone();
    let (pruned, removed) = prune_unneeded(&contents, &referenced);
    if removed > 0 {
        fs::write(path, pruned)?;
    }
    Ok(removed)
}

fn parse_exceptions(contents: &str) -> HashSet<String> {
    contents.lines().filter_map(parse_entry).collect()
}

fn parse_entry(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        None
    } else {
        normalize_slug(line)
    }
}

fn normalize_slug(value: &str) -> Option<String> {
    let value = value.split(['?', '#']).next().unwrap_or_default();
    let (_, value) = strip_locale_from_url(value);
    let value = value.strip_prefix('/').unwrap_or(value);
    let value = value.strip_prefix("docs/").unwrap_or(value);
    let value = value.trim_matches('/');
    (!value.is_empty()).then(|| url_to_folder_path(value).to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::{normalize_slug, parse_exceptions, prune_file_at, prune_unneeded};
    use std::collections::HashSet;

    #[test]
    fn parses_entries_ignoring_comments_and_blank_lines() {
        struct Case {
            name: &'static str,
            contents: &'static str,
            expected: &'static [&'static str],
        }
        let cases = [
            Case {
                name: "comments and blanks",
                contents: "# rationale\n\nWeb/API/Foo\n  # note\n Web/API/Bar \n",
                expected: &["web/api/foo", "web/api/bar"],
            },
            Case {
                name: "empty",
                contents: "\n # only a comment\n",
                expected: &[],
            },
        ];
        for case in cases {
            let actual = parse_exceptions(case.contents);
            assert_eq!(
                actual,
                case.expected.iter().map(|s| s.to_string()).collect(),
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn normalizes_locales_prefixes_queries_and_fragments() {
        struct Case {
            name: &'static str,
            url: &'static str,
            expected: Option<&'static str>,
        }
        let cases = [
            Case {
                name: "English URL",
                url: "/en-US/docs/Web/API/Foo?view=full#example",
                expected: Some("web/api/foo"),
            },
            Case {
                name: "translated URL",
                url: "/fr/docs/Web/API/Foo#example",
                expected: Some("web/api/foo"),
            },
            Case {
                name: "unlocalized path",
                url: "/docs/Web/API/Foo?x=1",
                expected: Some("web/api/foo"),
            },
            Case {
                name: "canonical case",
                url: "/de/docs/web/api/foo",
                expected: Some("web/api/foo"),
            },
            Case {
                name: "empty destination",
                url: "/fr/docs/#part",
                expected: None,
            },
        ];
        for case in cases {
            assert_eq!(
                normalize_slug(case.url).as_deref(),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn only_listed_missing_destinations_are_suppressed() {
        let exceptions = parse_exceptions("Web/API/Listed\n");
        struct Case {
            name: &'static str,
            url: &'static str,
            exists: bool,
            expected_suppression: bool,
        }
        let cases = [
            Case {
                name: "listed missing target",
                url: "/en-US/docs/Web/API/Listed#part",
                exists: false,
                expected_suppression: true,
            },
            Case {
                name: "unlisted missing target",
                url: "/fr/docs/Web/API/Other",
                exists: false,
                expected_suppression: false,
            },
            Case {
                name: "listed target that resolves",
                url: "/fr/docs/Web/API/Listed",
                exists: true,
                expected_suppression: false,
            },
        ];
        for case in cases {
            let actual = !case.exists
                && normalize_slug(case.url).is_some_and(|slug| exceptions.contains(&slug));
            assert_eq!(actual, case.expected_suppression, "{}", case.name);
        }
    }

    #[test]
    fn prunes_only_unused_entries_and_keeps_comments_and_spacing() {
        let contents = "# Group A rationale\nWeb/API/Keep\n\n# Group B rationale\nWeb/API/Remove\n";
        let remaining = HashSet::from(["web/api/keep".to_string()]);
        let (actual, removed) = prune_unneeded(contents, &remaining);
        assert_eq!(removed, 1);
        assert_eq!(actual, "# Group A rationale\nWeb/API/Keep\n");
    }

    #[test]
    fn keeps_rationale_for_surviving_entries_and_file_level_comments() {
        let contents = "# File note\n\n# Group rationale\nWeb/API/Keep\nWeb/API/Remove\n";
        let remaining = HashSet::from(["web/api/keep".to_string()]);
        let (actual, removed) = prune_unneeded(contents, &remaining);
        assert_eq!(removed, 1);
        assert_eq!(actual, "# File note\n\n# Group rationale\nWeb/API/Keep\n");
    }

    #[test]
    fn preserves_a_separated_file_header_when_all_groups_are_pruned() {
        let contents = "# Allowed broken links\n# Entries are reviewed exceptions.\n\n# Obsolete target rationale\nWeb/API/Removed\n";
        let (actual, removed) = prune_unneeded(contents, &HashSet::new());
        assert_eq!(removed, 1);
        assert_eq!(
            actual,
            "# Allowed broken links\n# Entries are reviewed exceptions.\n"
        );
    }

    #[test]
    fn references_from_multiple_locales_share_a_pruning_key() {
        let references = [
            "/en-US/docs/Web/API/Keep#anchor",
            "/fr/docs/Web/API/Keep?view=full",
        ]
        .into_iter()
        .filter_map(super::normalize_slug)
        .collect::<HashSet<_>>();
        let (actual, removed) = prune_unneeded("# Shared rationale\nWeb/API/Keep\n", &references);
        assert_eq!(removed, 0);
        assert_eq!(actual, "# Shared rationale\nWeb/API/Keep\n");
    }

    #[test]
    fn missing_exception_file_is_empty_for_pruning() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("_allowed-broken-links.txt");
        assert_eq!(prune_file_at(&path).unwrap(), 0);
        assert!(!path.exists());
    }
}
