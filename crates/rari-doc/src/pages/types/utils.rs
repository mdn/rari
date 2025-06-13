use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FmTempl {
    NoArgs(String),
    WithArgs { name: String, args: Vec<String> },
}

impl FmTempl {
    pub fn name(&self) -> &str {
        match self {
            FmTempl::NoArgs(name) => name,
            FmTempl::WithArgs { name, .. } => name,
        }
        .as_str()
    }
}

impl<'de> Deserialize<'de> for FmTempl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntryVisitor;

        impl<'de> Visitor<'de> for EntryVisitor {
            type Value = FmTempl;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("either a string or a single-key map to a list of strings")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FmTempl::NoArgs(s.to_owned()))
            }

            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(FmTempl::NoArgs(s))
            }

            fn visit_seq<A>(self, mut _seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Err(de::Error::custom(
                    "unexpected sequence; expected a map or string",
                ))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                if let Some((key, val)) = map.next_entry::<String, Vec<String>>()? {
                    if map.next_entry::<String, Vec<String>>()?.is_some() {
                        return Err(de::Error::custom("map has more than one key"));
                    }
                    Ok(FmTempl::WithArgs {
                        name: key,
                        args: val,
                    })
                } else {
                    Err(de::Error::custom("empty map is not allowed"))
                }
            }
        }

        deserializer.deserialize_any(EntryVisitor)
    }
}

impl Serialize for FmTempl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FmTempl::NoArgs(s) => serializer.serialize_str(s),
            FmTempl::WithArgs { name, args } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry(name, &args)?;
                map.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sidebar_fm() {
        let s = r#"["foobar", { "foo": ["bar", "2000"]}]"#;
        let j: Vec<FmTempl> = serde_json::from_str(s).unwrap();
        let sb = vec![
            FmTempl::NoArgs("foobar".to_string()),
            FmTempl::WithArgs {
                name: "foo".to_string(),
                args: vec!["bar".to_string(), "2000".to_string()],
            },
        ];
        assert_eq!(j, sb);
        let j = serde_yaml_ng::to_string(&sb).unwrap();
        assert_eq!(j, "- foobar\n- foo:\n  - bar\n  - '2000'\n");
    }
}
