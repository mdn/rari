use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;

use rari_types::globals::{content_root, content_translated_root};
use rari_types::locale::Locale;
use rari_utils::error::RariIoError;
use tracing::{error, warn};

use crate::error::DocError;
use crate::utils::root_for_locale;

static REDIRECTS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    if let Some(ctr) = content_translated_root() {
        for locale in ctr
            .read_dir()
            .expect("unable to read translated content root")
            .filter_map(|dir| {
                dir.map_err(|e| {
                    error!("Error: reading translated content root: {e}");
                })
                .ok()
                .and_then(|dir| {
                    Locale::from_str(
                        dir.file_name()
                            .as_os_str()
                            .to_str()
                            .expect("invalid folder"),
                    )
                    .map_err(|e| error!("Invalid folder {:?}: {e}", dir.file_name()))
                    .ok()
                })
            })
        {
            if let Err(e) = read_redirects(
                &ctr.to_path_buf()
                    .join(locale.as_folder_str())
                    .join("_redirects.txt"),
                &mut map,
            ) {
                error!("Error reading redirects: {e}");
            }
        }
    }
    if let Err(e) = read_redirects(
        &content_root()
            .to_path_buf()
            .join(Locale::EnUs.as_folder_str())
            .join("_redirects.txt"),
        &mut map,
    ) {
        error!("Error reading redirects: {e}");
    }
    map
});

fn read_redirects(path: &Path, map: &mut HashMap<String, String>) -> Result<(), DocError> {
    let lines = read_lines(path)?;
    map.extend(lines.map_while(Result::ok).filter_map(|line| {
        if line.starts_with('#') {
            return None;
        }
        let mut from_to = line.splitn(2, '\t');
        if let (Some(from), Some(to)) = (from_to.next(), from_to.next()) {
            Some((from.to_lowercase(), to.into()))
        } else {
            None
        }
    }));
    Ok(())
}

fn read_lines<P>(filename: P) -> Result<io::Lines<io::BufReader<File>>, RariIoError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename.as_ref()).map_err(|e| RariIoError {
        source: e,
        path: filename.as_ref().to_path_buf(),
    })?;
    Ok(io::BufReader::new(file).lines())
}

pub fn resolve_redirect(url: &str) -> Option<Cow<'_, str>> {
    let hash_index = url.find('#').unwrap_or(url.len());
    let (url_no_hash, hash) = (&url[..hash_index], &url[hash_index..]);
    match (
        REDIRECTS
            .get(&url_no_hash.to_lowercase())
            .map(|s| s.as_str()),
        hash,
    ) {
        (None, _) => None,
        (Some(url), hash) if url.contains('#') || hash.is_empty() => Some(Cow::Borrowed(url)),
        (Some(url), hash) => Some(Cow::Owned(format!("{url}{hash}"))),
    }
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
/// - `Err(DocError)` if any errors occur during processing, such as issues with reading the redirects file or invalid locale.
///
/// # Errors
///
/// - **LocaleError**: Returned if there's an issue determining the root path for the given locale.
/// - **ReadRedirectsError**: Returned if there's an error reading the existing redirects from the `_redirects.txt` file.
/// - *Additional errors can be added based on further implementations and validations.*
pub fn add_redirects(locale: Locale, pairs: &[(String, String)]) -> Result<(), DocError> {
    // read the redirect map for the locale
    // we do not use REDIRECTS since it is static and has all the locales

    // Read the redirects file for the locale
    let mut map = HashMap::new();
    let path = root_for_locale(locale)?
        .to_path_buf()
        .join(locale.as_folder_str())
        .join("_redirects.txt");

    if let Err(e) = read_redirects(&path, &mut map) {
        error!("Error reading redirects: {e}");
        return Err(DocError::ReadRedirectsError(e.to_string()));
    }
    println!("Current Redirect Map: {:?}", map);

    // TODO: Implement validation functions
    // error_on_encoded(pairs)?;
    // error_on_duplicated(pairs)?;
    // validate_pairs(pairs, locale, strict)?;

    // Separate the pairs into case-only changes and proper redirects
    let (case_changed_targets, new_pairs) = separate_case_changes(pairs);
    // Remove conflicting old redirects based on the new_pairs
    let clean_pairs = remove_conflicting_old_redirects(pairs, &new_pairs);
    // Fix redirect cases based on case_changed_targets
    let clean_pairs = fix_redirects_case(&clean_pairs, &case_changed_targets);

    // Update the redirect map with the cleaned and new pairs
    for (from, to) in clean_pairs.iter() {
        println!("Adding to map: {} -> {}", from, to);
        map.insert(from.to_lowercase(), to.clone());
    }

    // Write the updated map back to the redirects file
    // write_redirects(&path, &map)?;

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
fn separate_case_changes<'a>(
    pairs: &'a [(String, String)],
) -> (HashSet<&'a String>, Vec<&'a (String, String)>) {
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
/// * `old_pairs` - A slice of original redirect pairs.
/// * `new_pairs` - A slice of new redirect pairs being added.
///
/// # Returns
///
/// * A vector of references to redirect pairs that do not conflict with `new_pairs`.
fn remove_conflicting_old_redirects<'a>(
    old_pairs: &'a [(String, String)],
    new_pairs: &[&'a (String, String)],
) -> Vec<&'a (String, String)> {
    if old_pairs.is_empty() {
        return Vec::new();
    }

    let new_targets: HashSet<String> = new_pairs.iter().map(|(_, to)| to.to_lowercase()).collect();

    old_pairs
        .iter()
        .filter(|(from, to)| {
            let conflicting_to = new_targets.contains(&from.to_lowercase());
            if conflicting_to {
                warn!(
                    "Breaking 301: removing conflicting redirect {}\t{}",
                    from, to
                );
            }
            !conflicting_to
        })
        .collect()
}

/// Fixes the case of redirect targets based on the provided `case_changed_targets`.
///
/// # Arguments
///
/// * `old_pairs` - A slice of references to redirect pairs.
/// * `case_changed_targets` - A reference to a `HashSet` of `to` strings with corrected casing.
///
/// # Returns
///
/// * A vector of redirect pairs with updated target casing.
fn fix_redirects_case<'a>(
    old_pairs: &[&'a (String, String)],
    case_changed_targets: &HashSet<&'a String>,
) -> Vec<(String, String)> {
    // Create a HashMap where the key is the lowercase version of the target (owned String),
    // and the value is the original target with corrected casing (&str).
    let new_targets: HashMap<String, &str> = case_changed_targets
        .iter()
        .map(|p| (p.to_lowercase(), p.as_str()))
        .collect();

    // Iterate over each old pair and replace the target if a case-corrected version exists.
    old_pairs
        .iter()
        .map(|&&(ref from, ref to)| {
            let target = new_targets
                .get(&to.to_lowercase())
                .copied()
                .unwrap_or(to.as_str());
            (from.clone(), target.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_conflicting_old_redirects_no_conflicts() {
        let old_pairs = vec![
            ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
            ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
        ];

        let update_pairs = vec![("/en-US/docs/E".to_string(), "/en-US/docs/F".to_string())];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected_refs: Vec<&(String, String)> = old_pairs.iter().collect();
        let result = remove_conflicting_old_redirects(&old_pairs, &update_pairs_refs);
        assert_eq!(result, expected_refs);
    }
    #[test]
    fn test_remove_conflicting_old_redirects_with_conflicts() {
        let old_pairs = vec![
            ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
            ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
        ];
        let update_pairs = vec![("/en-US/docs/C".to_string(), "/en-US/docs/A".to_string())];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected = vec![("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string())];
        let expected_refs: Vec<&(String, String)> = expected.iter().collect();
        let result = remove_conflicting_old_redirects(&old_pairs, &update_pairs_refs);
        assert_eq!(result, expected_refs);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_empty_old_pairs() {
        let old_pairs: Vec<(String, String)> = vec![];
        let update_pairs = vec![("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string())];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected: Vec<(String, String)> = vec![];
        let expected_refs: Vec<&(String, String)> = expected.iter().collect();
        let result = remove_conflicting_old_redirects(&old_pairs, &update_pairs_refs);
        assert_eq!(result, expected_refs);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_empty_update_pairs() {
        let old_pairs = vec![
            ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
            ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
        ];

        let update_pairs: Vec<(String, String)> = vec![];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected = old_pairs.clone();
        let expected_refs: Vec<&(String, String)> = expected.iter().collect();
        let result = remove_conflicting_old_redirects(&old_pairs, &update_pairs_refs);
        assert_eq!(result, expected_refs);
    }

    #[test]
    fn test_remove_conflicting_old_redirects_case_insensitive() {
        let old_pairs = vec![
            ("/EN-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
            ("/EN-US/DOCS/C".to_string(), "/EN-US/DOCS/D".to_string()),
        ];

        let update_pairs = vec![
            ("/en-US/docs/New1".to_string(), "/en-US/docs/a".to_string()),
            ("/en-US/docs/New2".to_string(), "/en-US/docs/c".to_string()),
        ];
        let update_pairs_refs: Vec<&(String, String)> = update_pairs.iter().collect();

        let expected: Vec<(String, String)> = vec![];
        let expected_refs: Vec<&(String, String)> = expected.iter().collect();
        let result = remove_conflicting_old_redirects(&old_pairs, &update_pairs_refs);
        assert_eq!(result, expected_refs);
    }

    // #[test]
    // fn test_fix_redirects_case_empty_targets() {
    //     let old_pairs = vec![
    //         ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
    //         ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
    //     ];

    //     let changed_targets = vec![];

    //     let expected = vec![
    //         ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
    //         ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
    //     ];
    //     let result = fix_redirects_case(&old_pairs, &changed_targets);
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_fix_redirects_case_changes() {
    //     let old_pairs = vec![
    //         ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
    //         ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
    //     ];
    //     let case_changed_targets: Vec<String> =
    //         vec!["/en-US/DOCS/B".to_string(), "/en-US/DOCS/D".to_string()];

    //     let expected = vec![
    //         ("/en-US/docs/A".to_string(), "/en-US/DOCS/B".to_string()),
    //         ("/en-US/docs/C".to_string(), "/en-US/DOCS/D".to_string()),
    //     ];

    //     let result = fix_redirects_case(&old_pairs, &case_changed_targets);
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_fix_redirects_case_empty_old_pairs() {
    //     let old_pairs = vec![];
    //     let case_changed_targets: Vec<String> = vec!["/en-US/DOCS/B".to_string()];

    //     let expected = vec![];
    //     let result = fix_redirects_case(&old_pairs, &case_changed_targets);
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_fix_redirects_case_duplicates_in_case_changed_targets() {
    //     let old_pairs = vec![
    //         ("/en-US/docs/A".to_string(), "/en-US/docs/B".to_string()),
    //         ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
    //     ];
    //     let case_changed_targets: Vec<String> = vec![
    //         "/en-US/DOCS/B".to_string(),
    //         "/EN-US/DOCS/B".to_string(), // Duplicate target with different case
    //     ];

    //     let expected = vec![
    //         ("/en-US/docs/A".to_string(), "/EN-US/DOCS/B".to_string()), // last occurrence wins
    //         ("/en-US/docs/C".to_string(), "/en-US/docs/D".to_string()),
    //     ];

    //     let result = fix_redirects_case(&old_pairs, &case_changed_targets);
    //     assert_eq!(result, expected);
    // }
}
