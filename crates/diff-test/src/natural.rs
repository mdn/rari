use std::cmp::Ordering;

use serde::Serialize;

/// Diff map key ordered naturally, so array indices sort numerically
/// (`2` before `10`) rather than lexicographically.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct NaturalKey(pub String);

impl Ord for NaturalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        natural_cmp(&self.0, &other.0)
    }
}

impl PartialOrd for NaturalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn natural_cmp(lhs: &str, rhs: &str) -> Ordering {
    let (mut lhs, mut rhs) = (lhs.as_bytes(), rhs.as_bytes());
    while let (Some(l), Some(r)) = (lhs.first(), rhs.first()) {
        let order = if l.is_ascii_digit() && r.is_ascii_digit() {
            let (lhs_digits, lhs_rest) = split_digits(lhs);
            let (rhs_digits, rhs_rest) = split_digits(rhs);
            (lhs, rhs) = (lhs_rest, rhs_rest);
            lhs_digits
                .len()
                .cmp(&rhs_digits.len())
                .then_with(|| lhs_digits.cmp(rhs_digits))
        } else {
            (lhs, rhs) = (&lhs[1..], &rhs[1..]);
            l.cmp(r)
        };
        if order != Ordering::Equal {
            return order;
        }
    }
    lhs.len().cmp(&rhs.len())
}

fn split_digits(bytes: &[u8]) -> (&[u8], &[u8]) {
    bytes.split_at(bytes.iter().take_while(|b| b.is_ascii_digit()).count())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn natural_cmp_orders_path_keys() {
        struct Case {
            name: &'static str,
            lhs: &'static str,
            rhs: &'static str,
            expected: Ordering,
        }

        let cases = vec![
            Case {
                name: "identical keys",
                lhs: "doc.body.2.value.content",
                rhs: "doc.body.2.value.content",
                expected: Ordering::Equal,
            },
            Case {
                name: "shorter digit run sorts first",
                lhs: "1327.title",
                rhs: "13269.title",
                expected: Ordering::Less,
            },
            Case {
                name: "equal-length digit runs compare by value",
                lhs: "13270.title",
                rhs: "13269.title",
                expected: Ordering::Greater,
            },
            Case {
                name: "nested array index sorts numerically",
                lhs: "doc.body.2.value.content",
                rhs: "doc.body.10.value.content",
                expected: Ordering::Less,
            },
            Case {
                name: "prefix sorts before extension",
                lhs: "doc.body.2",
                rhs: "doc.body.2.value",
                expected: Ordering::Less,
            },
            Case {
                name: "non-digit segments compare bytewise",
                lhs: "doc.body",
                rhs: "doc.title",
                expected: Ordering::Less,
            },
            Case {
                name: "digit sorts before letter at same position",
                lhs: "doc.1",
                rhs: "doc.a",
                expected: Ordering::Less,
            },
        ];

        for Case {
            name,
            lhs,
            rhs,
            expected,
        } in cases
        {
            assert_eq!(natural_cmp(lhs, rhs), expected, "{name}");
        }
    }

    #[test]
    fn natural_key_map_serializes_numeric_path_keys_in_order() {
        let diff = BTreeMap::from([
            (NaturalKey("13269.title".into()), "third".to_string()),
            (NaturalKey("1327.title".into()), "first".to_string()),
            (NaturalKey("13270.title".into()), "fourth".to_string()),
        ]);

        assert_eq!(
            serde_json::to_string(&diff).unwrap(),
            r#"{"1327.title":"first","13269.title":"third","13270.title":"fourth"}"#
        );
    }
}
