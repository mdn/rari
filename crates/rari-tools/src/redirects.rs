use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;

use rari_types::globals::deny_warnings;
use rari_utils::concat_strs;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RedirectError {
    #[error("RedirectError: {0}")]
    Cycle(String),
    #[error("No cased version {0}")]
    NoCased(String),
}

// Transitive directed acyclic graph of all redirects.
// All redirects are expanded A -> B, B -> C becomes:
// A -> B, B -> C, A -> C and all cycles are removed.
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
                tracing::warn!("{msg}")
            }
            return Ok(None);
        }
        return transit(next, froms, dag);
    }
    Ok(Some(s.to_string()))
}

pub fn short_cuts<'a>(
    pairs: &'a [(&'a str, &'a str)],
) -> Result<Vec<(String, String)>, RedirectError> {
    let mut casing = pairs
        .iter()
        .flat_map(|(from, to)| {
            [
                (from.to_lowercase(), Cow::Borrowed(*from)),
                (to.to_lowercase(), Cow::Borrowed(*to)),
            ]
        })
        .collect::<HashMap<String, Cow<'_, str>>>();

    let lowercase_pairs: Vec<_> = pairs
        .iter()
        .map(|(from, to)| (from.to_lowercase(), to.to_lowercase()))
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
    // /en-US/docs/foo/bar     /en-US/docs/something#bar
    // /en-US/docs/foo     /en-US/docs/Web/something
    for (from, to) in pairs {
        if let Some((bare_to, hash)) = to.split_once('#') {
            let bare_to_lc = bare_to.to_lowercase();
            if let Some(redirected_to) = transitive_dag.get(&bare_to_lc) {
                let new_to = concat_strs!(redirected_to, "#", hash.to_lowercase().as_str());
                let redirected_to_cased = casing
                    .get(redirected_to.as_str())
                    .ok_or(RedirectError::NoCased(redirected_to.clone()))?;
                let new_to_cased = Cow::Owned(concat_strs!(redirected_to_cased, "#", hash));
                casing.insert(new_to.to_string(), new_to_cased);
                tracing::info!("Short cutting hashed redirect: {from} -> {new_to}");
                transitive_dag.insert(from.to_lowercase(), new_to);
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

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn simple_chain() {
            let pairs = vec![
                ("/en-US/docs/A", "/en-US/docs/B"),
                ("/en-US/docs/B", "/en-US/docs/C"),
            ];
            let result = short_cuts(&pairs).unwrap();
            let expected = vec![
                ("/en-US/docs/A".to_string(), "/en-US/docs/C".to_string()),
                ("/en-US/docs/B".to_string(), "/en-US/docs/C".to_string()),
            ];
            assert_eq!(result, expected)
        }

        #[test]
        fn a_equals_a() {
            let pairs = vec![
                ("/en-US/docs/A", "/en-US/docs/A"),
                ("/en-US/docs/B", "/en-US/docs/B"),
            ];
            let result = short_cuts(&pairs).unwrap();
            let expected: Vec<(String, String)> = vec![]; // empty result as expected
            assert_eq!(result, expected);
        }

        #[test]
        fn simple_cycle() {
            let pairs = vec![
                ("/en-US/docs/A", "/en-US/docs/B"),
                ("/en-US/docs/B", "/en-US/docs/A"),
            ];
            let result = short_cuts(&pairs).unwrap();
            let expected: Vec<(String, String)> = vec![]; // empty result due to cycle
            assert_eq!(result, expected);
        }

        #[test]
        fn hashes() {
            let pairs = vec![
                ("/en-US/docs/A", "/en-US/docs/B#Foo"),
                ("/en-US/docs/B", "/en-US/docs/C"),
            ];
            let result = short_cuts(&pairs).unwrap();
            let expected = vec![
                ("/en-US/docs/A".to_string(), "/en-US/docs/C#Foo".to_string()),
                ("/en-US/docs/B".to_string(), "/en-US/docs/C".to_string()),
            ];
            assert_eq!(result, expected);
        }
    }
}
