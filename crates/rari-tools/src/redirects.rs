use rari_doc::pages::page::{Page, PageLike};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use tracing::{error, warn};
use url::Url;

use rari_doc::resolve::{url_meta_from, UrlMeta};
use rari_doc::utils::root_for_locale;
use rari_types::globals::deny_warnings;
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use rari_utils::error::RariIoError;

use crate::error::{RedirectError, ToolError};

const REDIRECT_FILE_HEADER: &str = r#"# DO NOT EDIT THIS FILE MANUALLY.
# Use the CLI instead::
#
#    rari content add-redirect <fromURL> <toURL>
#
# FROM-URL	TO-URL
"#;

static FORBIDDEN_URL_SYMBOLS: [char; 2] = ['\t', '\n'];

/// Determines the final target of a redirect by traversing the redirect graph.
///
/// This function recursively follows redirects from a starting point `s` using the provided
/// directed acyclic graph (`dag`). It tracks the traversal path in `froms` to detect cycles.
/// If a cycle is detected and warnings are denied (see `Settings.deny_warnings`), it returns
/// an error. Otherwise, it logs a warning.
///
/// Transitive directed acyclic graph of all redirects.
/// All redirects are expanded A -> B, B -> C becomes:
/// A -> B, B -> C, A -> C and all cycles are removed.
///
/// The passed-in `froms` vector is used to track the sequence of URLs/paths traversed to detect
/// cycles. It is populated with all `from` paths that lead to the final `to` path.
///
/// # Parameters
///
/// - `s`: The starting URL/path as a string slice.
/// - `froms`: A mutable vector that tracks the sequence of URLs/paths traversed to detect cycles.
/// - `dag`: A reference to a `HashMap` representing the redirect graph, where each key is a source
///          URL/path and the corresponding value is its redirect target.
///
/// # Returns
///
/// - `Ok(Some(String))`: The final resolved target URL/path as a `String` if no further redirects are found.
/// - `Ok(None)`: Returns `None` if a redirect cycle is detected and warnings are allowed.
/// - `Err(RedirectError)`: An error of type `RedirectError` if a cycle is detected and warnings are denied.
///
/// # Errors
///
/// - Returns `RedirectError::Cycle` if a redirect cycle is detected and `deny_warnings()` is `true`.
fn transit<'a>(
    s: &'a str,
    froms: &mut Vec<&'a str>,
    dag: &'a HashMap<&'a str, &'a str>,
) -> Result<Option<String>, RedirectError> {
    let next = dag.get(s);
    if let Some(next) = next {
        froms.push(s);
        if froms.iter().any(|from| from == next) {
            let msg = format!("redirect cycle [{}] → {next}", froms.join(", "));
            if deny_warnings() {
                return Err(RedirectError::Cycle(msg));
            } else {
                warn!("{msg}")
            }
            return Ok(None);
        }
        return transit(next, froms, dag);
    }
    Ok(Some(s.to_string()))
}

/// Generates a list of transitive redirect shortcuts from a list of redirect pairs.
///
/// This function processes a list of `(from, to)` redirect pairs to create a transitive
/// mapping of redirects, resolving chains of redirects and detecting cycles. It also
/// handles case normalization and shortcuts hashed redirects to optimize the redirect paths.
///
/// # Parameters
///
/// - `pairs`: A slice of tuples, each containing a `from` and `to` string slice representing a redirect.
///
/// # Returns
///
/// - `Ok(Vec<(String, String)>)`: A vector of `(from, to)` pairs representing the transitive redirects.
/// - `Err(RedirectError)`: An error of type `RedirectError` if a cycle is detected or if case-sensitive
///    mappings are missing.
///
/// # Errors
///
/// - Returns `RedirectError::Cycle` if a redirect cycle is detected during processing.
/// - Returns `RedirectError::NoCased` if a case-sensitive mapping for a redirect target is missing.
pub fn short_cuts(
    pairs: &HashMap<impl AsRef<str>, impl AsRef<str>>,
) -> Result<Vec<(String, String)>, RedirectError> {
    let mut casing = pairs
        .iter()
        .flat_map(|(from, to)| {
            [
                (from.as_ref().to_lowercase(), Cow::Borrowed(from.as_ref())),
                (to.as_ref().to_lowercase(), Cow::Borrowed(to.as_ref())),
            ]
        })
        .collect::<HashMap<String, Cow<'_, str>>>();

    let lowercase_pairs: Vec<_> = pairs
        .iter()
        .map(|(from, to)| (from.as_ref().to_lowercase(), to.as_ref().to_lowercase()))
        .collect();

    let dag = lowercase_pairs
        .iter()
        .map(|(from, to)| (from.as_str(), to.as_str()))
        .collect();

    let mut transitive_dag = HashMap::new();

    for (from, _) in lowercase_pairs.iter() {
        let mut froms = vec![];
        let to = transit(from, &mut froms, &dag)?;
        if let Some(to) = to {
            for from in froms {
                transitive_dag.insert(from.to_string(), to.clone());
            }
        }
    }

    // We want to shortcut
    // /en-US/docs/foo/bar     /en-US/docs/foo#bar
    // /en-US/docs/foo     /en-US/docs/Web/something
    // to
    // /en-US/docs/foo/bar     /en-US/docs/Web/something#bar
    // /en-US/docs/foo     /en-US/docs/Web/something
    for (from, to) in pairs {
        if let Some((bare_to, hash)) = to.as_ref().split_once('#') {
            let bare_to_lc = bare_to.to_lowercase();
            if let Some(redirected_to) = transitive_dag.get(&bare_to_lc) {
                let new_to = concat_strs!(redirected_to, "#", hash.to_lowercase().as_str());
                let redirected_to_cased = casing
                    .get(redirected_to.as_str())
                    .ok_or(RedirectError::NoCased(redirected_to.clone()))?;
                let new_to_cased = Cow::Owned(concat_strs!(redirected_to_cased, "#", hash));
                casing.insert(new_to.to_string(), new_to_cased);
                tracing::info!(
                    "Short cutting hashed redirect: {} -> {new_to}",
                    from.as_ref()
                );
                transitive_dag.insert(from.as_ref().to_lowercase(), new_to);
            }
        }
    }

    // Collect and restore cases!
    let mut transitive_pairs: Vec<(String, String)> = transitive_dag
        .into_iter()
        .map(|(from, to)| {
            (
                casing
                    .get(from.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(from),
                casing.get(to.as_str()).map(|s| s.to_string()).unwrap_or(to),
            )
        })
        .collect();
    transitive_pairs.sort_by(|(a_from, a_to), (b_from, b_to)| match a_from.cmp(b_from) {
        Ordering::Equal => a_to.cmp(b_to),
        x => x,
    });
    Ok(transitive_pairs)
}

/// Adds new redirect pairs to the existing redirects for a specified locale.
///
/// This function performs the following steps:
///
/// 1. **Reads Existing Redirects**: It reads the current redirects from a `_redirects.txt` file specific to the provided `locale`.
/// 2. **Validates Redirect Pairs**: It separates the incoming `pairs` into those that only change the case of the target and those that represent actual redirects.
/// 3. **Removes Conflicting Redirects**: It filters out any old redirects that conflict with the new redirects being added.
/// 4. **Fixes Redirect Cases**: It updates the cases of redirect targets based on the provided `case_changed_targets`.
/// 5. **Updates Redirect Map**: It integrates the new and updated redirects into the existing redirect map.
/// 6. **Writes Back to File**: It writes the updated redirects back to the `_redirects.txt` file.
///
/// # Parameters
///
/// - `locale`: The `Locale` for which the redirects are to be added. This determines the specific `_redirects.txt` file to be read and updated.
/// - `pairs`: A slice of tuples, where each tuple consists of a `from` and `to` string representing the redirect paths.
///
/// # Returns
///
/// - `Ok(())` if the redirects are successfully added and processed.
/// - `Err(ToolError)` if any errors occur during processing, such as issues with reading the redirects file or invalid locale.
///
/// # Errors
///
/// - **LocaleError**: Returned if there's an issue determining the root path for the given locale.
/// - **ReadRedirectsError**: Returned if there's an error reading the existing redirects from the `_redirects.txt` file.
/// - *Additional errors can be added based on further implementations and validations.*
pub fn add_redirects(locale: Locale, update_pairs: &[(String, String)]) -> Result<(), ToolError> {
    // read the redirect map for the locale
    // we do not use REDIRECTS since it is static and has all the locales

    // Read the redirects file for the locale and populate the map.
    let mut pairs = HashMap::new();
    let path = root_for_locale(locale)?
        .to_path_buf()
        .join(locale.as_folder_str())
        .join("_redirects.txt");

    if let Err(e) = read_redirects_raw(&path, &mut pairs) {
        error!("Error reading redirects: {e}");
        return Err(ToolError::ReadRedirectsError(e.to_string()));
    }

    // Separate the pairs into case-only changes and proper redirects
    let (case_changed_targets, new_pairs) = separate_case_changes(update_pairs);

    // Remove conflicting old redirects based on the new_pairs
    remove_conflicting_old_redirects(&mut pairs, &new_pairs);
    // Fix redirect cases based on case_changed_targets
    let mut clean_pairs = fix_redirects_case(pairs, &case_changed_targets);

    // Add the new pairs to the clean_pairs
    for (from, to) in new_pairs {
        clean_pairs.insert(from.to_string(), to.to_string());
    }

    let clean_pairs: HashMap<String, String> = short_cuts(&clean_pairs)?.into_iter().collect();

    validate_pairs(&clean_pairs, &locale)?;

    // Write the updated map back to the redirects file
    write_redirects(&path, &clean_pairs)?;

    Ok(())
}

/// Validates a list of redirect pairs.
///
/// Iterates through each `(from, to)` pair and validates both URLs based on the locale.
///
/// # Arguments
///
/// * `pairs` - A `HashMap` of redirect pairs.
/// * `locale` - A reference to the `Locale`
///
/// # Returns
///
/// * `Ok(())` if all pairs are valid.
/// * `Err(ToolError)` if any pair is invalid.
fn validate_pairs(pairs: &HashMap<String, String>, locale: &Locale) -> Result<(), ToolError> {
    for (from, to) in pairs {
        validate_from_url(from, locale)?;
        validate_to_url(to, locale)?;
    }
    Ok(())
}

/// Validates the 'from' URL in a redirect pair.
///
/// Ensures that the URL:
/// - Starts with a `/`.
/// - Has the correct locale prefix.
/// - Contains `/docs/`.
/// - Does not contain forbidden symbols.
/// - Does not resolve to an existing file/folder.
///
/// # Arguments
///
/// * `url` - The 'from' URL to validate.
/// * `locale` - The expected `Locale`.
///
/// # Returns
///
/// * `Ok(())` if the URL is valid.
/// * `Err(ToolError)` if the URL is invalid.
fn validate_from_url(url: &str, locale: &Locale) -> Result<(), ToolError> {
    let url = url.to_lowercase();
    if !url.starts_with('/') {
        return Err(ToolError::InvalidRedirectFromURL(format!(
            "From-URL must start with a '/' but was '{}'",
            url
        )));
    }

    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 4 {
        return Err(ToolError::InvalidRedirectFromURL(format!(
            "From-URL '{}' does not have enough parts for locale validation.",
            url
        )));
    }

    let from_locale = parts[1];
    if from_locale != locale.as_url_str().to_lowercase() {
        return Err(ToolError::InvalidRedirectFromURL(format!(
            "From-URL has locale '{}' which does not match expected locale '{}'. URL: '{}'",
            from_locale, locale, url
        )));
    }

    if parts[2] != "docs" {
        return Err(ToolError::InvalidRedirectFromURL(format!(
            "From-URL '{}' must contain '/docs/'",
            url
        )));
    }

    check_url_invalid_symbols(&url)?;

    // Check for existing file/folder, commented for now
    if let Ok(page) = Page::from_url(&url) {
        return Err(ToolError::InvalidRedirectFromURL(format!(
            "From-URL '{}' resolves to an existing folder at '{}'.",
            url,
            page.path().display()
        )));
    }

    Ok(())
}

/// Validates the 'to' URL in a redirect pair.
///
/// Ensures that the URL:
/// - Is either a vanity URL, an external HTTPS URL, or an internal URL that resolves correctly.
/// - Does not contain forbidden symbols.
/// - Has a valid locale prefix.
/// - Resolves to an existing file
///
/// # Arguments
///
/// * `url` - The 'to' URL to validate.
/// * `locale` - The expected `Locale`.
///
/// # Returns
///
/// * `Ok(())` if the URL is valid.
/// * `Err(ToolError)` if the URL is invalid.
fn validate_to_url(url: &str, locale: &Locale) -> Result<(), ToolError> {
    if is_vanity_redirect_url(url) {
        return Ok(());
    }

    if url.contains("://") {
        // External URL, validate protocol
        let parsed_url =
            Url::parse(url).map_err(|e| ToolError::InvalidRedirectToURL(e.to_string()))?;
        if parsed_url.scheme() != "https" {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "We only redirect to 'https://', but got '{}://'",
                parsed_url.scheme()
            )));
        }
    } else if url.starts_with('/') {
        // Internal URL, perform validations

        check_url_invalid_symbols(url)?;
        validate_url_locale(url)?;

        // Split by '#', take the bare URL
        let bare_url = url.split('#').next().unwrap_or("");

        let parts: Vec<&str> = bare_url.split('/').collect();
        if parts.len() < 2 {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "To-URL '{}' does not have enough parts for locale validation.",
                url
            )));
        }

        let to_locale = parts[1];
        if to_locale != locale.as_url_str().to_lowercase() {
            // Different locale, no need to check path
            return Ok(());
        }

        let UrlMeta {
            folder_path: path, ..
        } = url_meta_from(url)?;
        let path = root_for_locale(*locale)?
            .join(locale.as_folder_str())
            .join(path);
        if !path.exists() {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "To-URL '{}' resolves to a non-existing file/folder at '{}'.",
                url,
                path.display()
            )));
        }
    } else {
        return Err(ToolError::InvalidRedirectToURL(format!(
            "To-URL '{}' has to be external (https://) or start with '/'.",
            url
        )));
    }

    Ok(())
}

fn validate_url_locale(url: &str) -> Result<(), ToolError> {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 3 {
        return Err(ToolError::InvalidRedirectToURL(format!(
            "To-URL '{}' does not have enough parts for locale validation.",
            url
        )));
    }

    let to_locale = parts[1];
    if Locale::from_str(to_locale).is_err() {
        return Err(ToolError::InvalidRedirectToURL(format!(
            "Locale prefix '{}' in To-URL '{}' is not valid.",
            to_locale, url
        )));
    }

    Ok(())
}

fn is_vanity_redirect_url(url: &str) -> bool {
    url.strip_prefix('/')
        .and_then(|url| url.strip_suffix('/'))
        .map(|locale_str| Locale::from_str(locale_str).is_ok())
        .unwrap_or_default()
}

fn check_url_invalid_symbols(url: &str) -> Result<(), ToolError> {
    if let Some(symbol_index) = url.find(FORBIDDEN_URL_SYMBOLS) {
        if let Some(escaped_symbol) = &url[symbol_index..]
            .chars()
            .next()
            .map(|c| c.escape_default())
        {
            return Err(ToolError::InvalidRedirectToURL(format!(
                "URL '{url}' contains forbidden symbol '{escaped_symbol}'."
            )));
        }
    }
    Ok(())
}

/// Reads redirect pairs from a file and populates the provided `HashMap`.
///
/// This function processes a file located at the specified `path`, where each line in the file
/// represents a redirect pair in the format `from\tto`. Lines that start with the `#` character
/// are treated as comments and are ignored. Only lines containing exactly two fields separated
/// by a tab (`\t`) are considered valid and are inserted into the map.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that points to the redirects file.
/// * `map` - A mutable reference to a `HashMap<String, String>` where the redirect pairs will
///           be stored. The `from` path serves as the key, and the `to` path serves as the value.
///
/// # Returns
///
/// * `Ok(())` if the redirects are successfully read and inserted into the `map`.
/// * `Err(ToolError)` if an error occurs while reading the file or processing its contents.
///
/// # Errors
///
/// This function will return a `ToolError` in the following situations:
///
/// * **File Read Error:** If the file at the specified `path` cannot be opened or read.
pub(crate) fn read_redirects_raw(
    path: &Path,
    map: &mut HashMap<String, String>,
) -> Result<(), ToolError> {
    let lines = read_lines(path)?;
    map.extend(lines.map_while(Result::ok).filter_map(|line| {
        if line.starts_with('#') {
            return None;
        }
        let mut from_to = line.trim().splitn(2, '\t');
        if let (Some(from), Some(to)) = (from_to.next(), from_to.next()) {
            Some((from.trim().into(), to.trim().into()))
        } else {
            None
        }
    }));
    Ok(())
}

/// Writes the redirects from the HashMap to the specified file path.
///
/// Each redirect is written in the format: `from to`.
///
/// # Arguments
///
/// * `path` - The path to the `_redirects.txt` file.
/// * `map` - A reference to the HashMap containing the redirects.
///
/// # Returns
///
/// * `Ok(())` if the file is written successfully.
/// * `Err(String)` with an error message if writing fails.
fn write_redirects(path: &Path, map: &HashMap<String, String>) -> Result<(), ToolError> {
    let file = File::create(path)?;
    let mut buffed = BufWriter::new(file);

    // Sort the Map by making a BTreeMap from the map.
    let sorted_map: BTreeMap<_, _> = map.iter().collect();

    // Write the file header:
    buffed.write_all(REDIRECT_FILE_HEADER.as_bytes())?;

    // Write each redirect pair to the file.
    for (from, to) in sorted_map.iter() {
        buffed.write_all(from.as_bytes())?;
        buffed.write_all(b"\t")?;
        buffed.write_all(to.as_bytes())?;
        buffed.write_all(b"\n")?;
    }

    Ok(())
}

/// Separates redirect pairs into case-only changes and proper redirects.
///
/// # Arguments
///
/// * `pairs` - A slice of redirect pairs.
///
/// # Returns
///
/// * A tuple containing:
///   * A `HashSet` of references to `to` strings that only change case.
///   * A vector of references to redirect pairs that represent proper redirects.
fn separate_case_changes(pairs: &[(String, String)]) -> (HashSet<&String>, Vec<&(String, String)>) {
    let mut case_changed_targets = HashSet::new();
    let mut new_pairs = Vec::new();

    for pair in pairs.iter() {
        if pair.0.to_lowercase() == pair.1.to_lowercase() {
            case_changed_targets.insert(&pair.1);
        } else {
            new_pairs.push(pair);
        }
    }

    (case_changed_targets, new_pairs)
}

/// Removes conflicting old redirects based on the new update pairs.
///
/// A redirect is considered conflicting if its `from` (case-insensitive) matches any `to` in `new_pairs`.
///
/// # Arguments
///
/// * `old_pairs` - A HashMap of the original Entries to be retained.
/// * `new_pairs` - A slice of new redirect pairs being added.
///
fn remove_conflicting_old_redirects(
    old_pairs: &mut HashMap<String, String>,
    new_pairs: &[&(String, String)],
) {
    if old_pairs.is_empty() {
        return;
    }

    // Collect the lowercased 'to' values from new_pairs into a HashSet for quick lookup
    let new_targets: HashSet<String> = new_pairs.iter().map(|(_, to)| to.to_lowercase()).collect();

    // Iterate over old_pairs, taking ownership of each (from, to) pair
    old_pairs.retain(|from, to| {
        let from_lower = from.to_lowercase();
        let retain = !new_targets.contains(&from_lower);
        if !retain {
            // Log a warning if there's a conflict
            warn!(
                "Breaking 301: removing conflicting redirect {}\t{}",
                from, to
            );
            // Skip inserting this pair into filtered_old
        }
        retain
    });
}

/// Fixes the casing of redirect targets based on a set of case-changed targets.
///
/// This function takes ownership of the existing redirects (`old_pairs`), applies
/// casing corrections to the targets as specified in `case_changed_targets`,
/// and returns a new `HashMap` with the updated redirects.
///
/// # Parameters
/// - `old_pairs`: A `HashMap` containing the existing redirects. Ownership is taken.
/// - `case_changed_targets`: A reference to a `HashSet` containing references to
///   target URLs with corrected casing.
///
/// # Returns
/// A new `HashMap<String, String>` containing the redirects with corrected target casing.
fn fix_redirects_case(
    old_pairs: HashMap<String, String>,
    case_changed_targets: &HashSet<&String>,
) -> HashMap<String, String> {
    if old_pairs.is_empty() {
        return HashMap::new();
    }

    // Create a HashMap where the key is the lowercase version of the target,
    // and the value is a reference to the corrected case version of the target.
    let new_targets: HashMap<String, &String> = case_changed_targets
        .iter()
        .map(|&target| (target.to_lowercase(), target))
        .collect();

    // Process each (from, to) pair, applying case corrections where necessary
    old_pairs
        .into_iter()
        .map(|(from, to)| {
            let to_lower = to.to_lowercase();
            match new_targets.get(&to_lower) {
                Some(&corrected_to) if &to != corrected_to => {
                    // Log a warning if the casing has changed
                    warn!("Fixing redirect target case: {} -> {}", to, corrected_to);
                    // Insert the pair with the corrected 'to'
                    (from, corrected_to.clone())
                }
                _ => {
                    // Insert the pair as is (no change needed)
                    (from, to)
                }
            }
        })
        .collect()
}

fn read_lines<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, ToolError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename.as_ref()).map_err(|e| RariIoError {
        source: e,
        path: filename.as_ref().to_path_buf(),
    })?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn test_remove_conflicting_old_redirects_no_conflicts() {
        let mut old_pairs = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);

        let update_pairs = [(s("/en-US/docs/E"), s("/en-US/docs/F"))];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected_refs = old_pairs.clone();
        remove_conflicting_old_redirects(&mut old_pairs, &update_pairs_refs);
        assert_eq!(old_pairs, expected_refs);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_with_conflicts() {
        let mut old_pairs = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);
        let update_pairs = [(s("/en-US/docs/C"), s("/en-US/docs/A"))];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected = HashMap::from([(s("/en-US/docs/C"), s("/en-US/docs/D"))]);
        remove_conflicting_old_redirects(&mut old_pairs, &update_pairs_refs);
        assert_eq!(old_pairs, expected);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_empty_old_pairs() {
        let mut old_pairs: HashMap<String, String> = HashMap::new();
        let update_pairs = [(s("/en-US/docs/A"), s("/en-US/docs/B"))];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected = HashMap::new();
        remove_conflicting_old_redirects(&mut old_pairs, &update_pairs_refs);
        assert_eq!(old_pairs, expected);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_empty_update_pairs() {
        let mut old_pairs = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);

        let update_pairs = [];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);
        remove_conflicting_old_redirects(&mut old_pairs, &update_pairs_refs);
        assert_eq!(old_pairs, expected);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_case_insensitive() {
        let mut old_pairs = HashMap::from([
            (s("/EN-US/docs/A"), s("/en-US/docs/B")),
            (s("/EN-US/DOCS/C"), s("/EN-US/DOCS/D")),
        ]);

        let update_pairs = [
            (s("/en-US/docs/New1"), s("/en-US/docs/a")),
            (s("/en-US/docs/New2"), s("/en-US/docs/c")),
        ];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected = HashMap::new();
        remove_conflicting_old_redirects(&mut old_pairs, &update_pairs_refs);
        assert_eq!(old_pairs, expected);
    }

    #[test]
    fn test_fix_redirects_case_empty_targets() {
        let old_pairs = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);

        let changed_targets = HashSet::new();

        let expected = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);
        let result = fix_redirects_case(old_pairs, &changed_targets);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fix_redirects_case_changes() {
        let old_pairs = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ]);

        let case_changed_targets = HashSet::from([s("/en-US/DOCS/B"), s("/en-US/DOCS/D")]);
        let case_changed_targets_ref = case_changed_targets.iter().collect();

        let expected = HashMap::from([
            (s("/en-US/docs/A"), s("/en-US/DOCS/B")),
            (s("/en-US/docs/C"), s("/en-US/DOCS/D")),
        ]);

        let result = fix_redirects_case(old_pairs, &case_changed_targets_ref);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_fix_redirects_case_empty_old_pairs() {
        let old_pairs = HashMap::new();
        let case_changed_targets = HashSet::from([s("/en-US/DOCS/B")]);
        let case_changed_targets_ref = case_changed_targets.iter().collect();

        let expected = HashMap::new();
        let result = fix_redirects_case(old_pairs, &case_changed_targets_ref);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_separate_case_changes_no_changes() {
        let pairs = vec![
            (s("/en-US/docs/A"), s("/en-US/docs/B")),
            (s("/en-US/docs/C"), s("/en-US/docs/D")),
        ];

        let (case_changed, proper_redirects) = separate_case_changes(&pairs);

        // All redirects are proper; no case changes.
        assert!(case_changed.is_empty(), "Expected no case changes");

        // All pairs should be in proper_redirects.
        assert_eq!(
            proper_redirects.len(),
            pairs.len(),
            "All pairs should be proper redirects"
        );
        for pair in &pairs {
            assert!(
                proper_redirects.contains(&pair),
                "Proper redirects should contain {:?}",
                pair
            );
        }
    }

    #[test]
    fn test_separate_case_changes() {
        let pairs = vec![
            (s("/en-US/docs/A"), s("/en-US/docs/a")),      // Case change
            (s("/en-US/docs/B"), s("/en-US/docs/b.html")), // Proper redirect
            (s("/en-US/docs/C"), s("/en-US/docs/C")),      // no actual change
            (s("/en-US/docs/D"), s("/en-US/docs/d.html")), // Proper redirect
        ];

        let (case_changed, proper_redirects) = separate_case_changes(&pairs);

        // There should be 2 case changes.
        assert_eq!(case_changed.len(), 2, "Expected 2 case changed targets");

        assert!(
            case_changed.contains(&&s("/en-US/docs/a")),
            "Case changed should contain '/en-US/docs/a'"
        );
        assert!(
            case_changed.contains(&&s("/en-US/docs/C")),
            "Case changed should contain '/en-US/docs/C'"
        );

        // There should be 2 proper redirects.
        assert_eq!(proper_redirects.len(), 2, "Expected 2 proper redirects");

        assert!(
            proper_redirects.contains(&&(
                "/en-US/docs/B".to_string(),
                "/en-US/docs/b.html".to_string()
            )),
            "Proper redirects should contain ('/en-US/docs/B', '/en-US/docs/b.html')"
        );

        assert!(
            proper_redirects.contains(&&(
                "/en-US/docs/D".to_string(),
                "/en-US/docs/d.html".to_string()
            )),
            "Proper redirects should contain ('/en-US/docs/D', '/en-US/docs/d.html')"
        );
    }

    #[test]
    fn test_validate_from_url_happy_path() {
        let url = "/en-US/docs/A";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_from_url_url_does_not_start_with_slash() {
        let url = "en-US/docs/A";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_url_url_has_insufficient_parts() {
        let url = "/en-US/docs";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_url_locale_mismatch() {
        let url = "/pt-BR/docs/A";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_url_missing_docs_segment() {
        let url = "/en-US/A";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_url_invalid_locale_prefix() {
        let url = "/hu/docs/A";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_from_url_forbidden_symbol() {
        let url = "/en-US/docs/\nA";
        let locale = Locale::EnUs;
        let result = validate_from_url(url, &locale);
        assert!(result.is_err());
    }

    #[test]
    fn simple_chain() {
        let pairs = [
            ("/en-US/docs/A", "/en-US/docs/B"),
            ("/en-US/docs/B", "/en-US/docs/C"),
        ]
        .into_iter()
        .collect();
        let result = short_cuts(&pairs).unwrap();
        let expected = vec![
            ("/en-US/docs/A".to_string(), "/en-US/docs/C".to_string()),
            ("/en-US/docs/B".to_string(), "/en-US/docs/C".to_string()),
        ];
        assert_eq!(result, expected)
    }

    #[test]
    fn a_equals_a() {
        let pairs = [
            ("/en-US/docs/A", "/en-US/docs/A"),
            ("/en-US/docs/B", "/en-US/docs/B"),
        ]
        .into_iter()
        .collect();
        let result = short_cuts(&pairs).unwrap();
        let expected: Vec<(String, String)> = vec![]; // empty result as expected
        assert_eq!(result, expected);
    }

    #[test]
    fn simple_cycle() {
        let pairs = [
            ("/en-US/docs/A", "/en-US/docs/B"),
            ("/en-US/docs/B", "/en-US/docs/A"),
        ]
        .into_iter()
        .collect();
        let result = short_cuts(&pairs).unwrap();
        let expected: Vec<(String, String)> = vec![]; // empty result due to cycle
        assert_eq!(result, expected);
    }

    #[test]
    fn hashes() {
        let pairs = [
            ("/en-US/docs/A", "/en-US/docs/B#Foo"),
            ("/en-US/docs/B", "/en-US/docs/C"),
        ]
        .into_iter()
        .collect();
        let result = short_cuts(&pairs).unwrap();
        let expected = vec![
            ("/en-US/docs/A".to_string(), "/en-US/docs/C#Foo".to_string()),
            ("/en-US/docs/B".to_string(), "/en-US/docs/C".to_string()),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_transit_success() {
        let mut froms = Vec::new();
        let mut dag = HashMap::new();
        dag.insert("/a", "/b");
        dag.insert("/b", "/c");
        dag.insert("/c", "/d");

        let result = transit("/a", &mut froms, &dag);
        assert!(result.is_ok());
        let result = result.unwrap().unwrap();
        assert_eq!(result, "/d".to_string());
        assert_eq!(froms, vec!["/a", "/b", "/c"]);

        let mut froms = Vec::new();
        let mut dag = HashMap::new();
        dag.insert("/a", "/b");
        dag.insert("/c", "/d");
        dag.insert("/e", "/f");

        let result = transit("/a", &mut froms, &dag);
        assert!(result.is_ok());
        let result = result.unwrap().unwrap();
        assert_eq!(result, "/b".to_string());
        assert_eq!(froms, vec!["/a"]);
    }

    #[test]
    fn test_transit_failure() {
        let mut froms = Vec::new();
        let mut dag = HashMap::new();
        dag.insert("/a", "/b");
        dag.insert("/b", "/c");
        dag.insert("/c", "/a");

        let result = transit("/a", &mut froms, &dag);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(froms, vec!["/a", "/b", "/c"]);
    }

    #[test]
    fn test_short_cuts_shortcut_hashed_redirect() {
        // Define the input redirect pairs
        let redirect_pairs = [
            ("/en-US/docs/foo/bar", "/en-US/docs/foo#bar"),
            ("/en-US/docs/foo", "/en-US/docs/Web/something"),
        ]
        .into_iter()
        .collect();

        // Call the short_cuts function with the input pairs
        let result = short_cuts(&redirect_pairs);

        // Ensure the function executed successfully
        assert!(result.is_ok(), "short_cuts should return Ok");

        // Extract the transitive redirect pairs
        let transitive_redirects = result.unwrap();

        // Define the expected transitive redirects
        let expected_redirects = vec![
            (
                "/en-US/docs/foo/bar".to_string(),
                "/en-US/docs/Web/something#bar".to_string(),
            ),
            (
                "/en-US/docs/foo".to_string(),
                "/en-US/docs/Web/something".to_string(),
            ),
        ];

        // Sort both vectors to ensure order doesn't affect the comparison
        let mut sorted_transitive = transitive_redirects.clone();
        sorted_transitive.sort();

        let mut sorted_expected = expected_redirects.clone();
        sorted_expected.sort();

        // Assert that the transitive redirects match the expected redirects
        assert_eq!(
            sorted_transitive, sorted_expected,
            "The transitive redirects do not match the expected shortcuts."
        );
    }
}
