use std::collections::BTreeMap;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use semver::VersionReq;
use serde::{Deserialize, Deserializer, Serialize};

use crate::locale::Locale;

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Deps {
    #[serde(rename = "@mdn/browser-compat-data")]
    pub bcd: VersionReq,
    #[serde(rename = "browser-specs")]
    pub browser_specs: VersionReq,
    #[serde(rename = "mdn-data")]
    pub mdn_data: VersionReq,
    #[serde(rename = "web-features")]
    pub web_features: VersionReq,
    #[serde(rename = "web-specs")]
    pub web_specs: VersionReq,
    #[serde(rename = "@webref/css")]
    pub webref_css: VersionReq,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DepsPackageJson {
    dependencies: Deps,
}

const PINNED_DEPS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../deps/package.json"
));

impl Deps {
    pub fn new() -> Self {
        serde_json::from_str::<DepsPackageJson>(PINNED_DEPS)
            .expect("embedded deps pins (deps/package.json) must be valid")
            .dependencies
    }
}

impl Default for Deps {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Settings {
    pub content_root: PathBuf,
    pub content_translated_root: Option<PathBuf>,
    pub translated_content_sources: BTreeMap<Locale, TranslatedContentSource>,
    pub build_out_root: Option<PathBuf>,
    pub blog_root: Option<PathBuf>,
    pub generic_content_root: Option<PathBuf>,
    pub curriculum_root: Option<PathBuf>,
    pub contributor_spotlight_root: Option<PathBuf>,
    pub deny_warnings: bool,
    pub cache_content: bool,
    pub base_url: String,
    pub live_samples_base_url: String,
    pub interactive_examples_base_url: String,
    pub additional_locales_for_generics_and_spas: Vec<Locale>,
    #[serde(deserialize_with = "deserialize_optional_translated_locales")]
    pub optional_translated_locales: Vec<Locale>,
    pub reader_ignores_gitignore: bool,
    pub data_issues: bool,
    pub json_issues: bool,
    pub json_live_samples: bool,
    pub blog_unpublished: bool,
    pub blog_pagination: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TranslatedContentSource {
    pub root: PathBuf,
    pub repository: String,
}

fn deserialize_optional_translated_locales<'de, D>(deserializer: D) -> Result<Vec<Locale>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|locale| locale.trim().parse().map_err(serde::de::Error::custom))
        .collect()
}

impl Settings {
    fn validate_optional_translated_locales(&mut self) -> Result<(), ConfigError> {
        if self.optional_translated_locales.contains(&Locale::EnUs) {
            return Err(ConfigError::Message(
                "optional_translated_locales cannot contain en-US".into(),
            ));
        }
        let mut seen = std::collections::HashSet::new();
        self.optional_translated_locales
            .retain(|locale| seen.insert(*locale));
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn validate(mut self) -> Self {
        self.content_root =
            std::fs::canonicalize(self.content_root).expect("CONTENT_ROOT is not a valid path");

        self.content_translated_root =
            self.content_translated_root.map(|translated_content_root| {
                std::fs::canonicalize(translated_content_root)
                    .expect("CONTENT_TRANSLATED_ROOT is not a valid path")
            });
        for (locale, source) in &mut self.translated_content_sources {
            assert_ne!(*locale, Locale::EnUs, "en-US uses CONTENT_ROOT");
            assert!(
                !source.repository.is_empty(),
                "repository must not be empty"
            );
            source.root = match std::fs::canonicalize(&source.root) {
                Ok(root) => root,
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        && self.optional_translated_locales.contains(locale) =>
                {
                    if source.root.is_absolute() {
                        source.root.clone()
                    } else {
                        std::env::current_dir()
                            .expect("unable to resolve optional translated-content source")
                            .join(&source.root)
                    }
                }
                Err(error) => panic!("translated content root for {locale} is not valid: {error}"),
            };
            if self.content_translated_root.as_ref() == Some(&source.root) {
                assert_eq!(
                    source.repository, "translated-content",
                    "repository for {locale} conflicts with CONTENT_TRANSLATED_ROOT"
                );
            }
            match std::fs::metadata(source.root.join(locale.as_folder_str())) {
                Ok(metadata) => assert!(
                    metadata.is_dir(),
                    "translated content root for {locale} has no locale directory"
                ),
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        && self.optional_translated_locales.contains(locale) => {}
                Err(error) => {
                    panic!("translated content root for {locale} has no locale directory: {error}")
                }
            }
        }
        for (locale, source) in &self.translated_content_sources {
            for (other_locale, other_source) in &self.translated_content_sources {
                if locale != other_locale && source.root == other_source.root {
                    assert_eq!(
                        source.repository, other_source.repository,
                        "locales sharing a translated-content root must use the same repository"
                    );
                }
            }
        }
        self
    }

    #[cfg(target_arch = "wasm32")]
    fn validate(self) -> Self {
        self
    }

    #[cfg(feature = "testing")]
    pub fn new() -> Result<Self, ConfigError> {
        unsafe {
            std::env::set_var(
                "CONTENT_ROOT",
                std::env::var("TESTING_CONTENT_ROOT").unwrap(),
            );
            std::env::set_var(
                "CONTENT_TRANSLATED_ROOT",
                std::env::var("TESTING_CONTENT_TRANSLATED_ROOT").unwrap(),
            );
            std::env::set_var("BLOG_ROOT", std::env::var("TESTING_BLOG_ROOT").unwrap());
            std::env::set_var(
                "CACHE_CONTENT",
                std::env::var("TESTING_CACHE_CONTENT").unwrap(),
            );
            std::env::set_var(
                "READER_IGNORES_GITIGNORE",
                std::env::var("TESTING_READER_IGNORES_GITIGNORE").unwrap(),
            );
        }
        Self::new_internal()
    }
    #[cfg(not(feature = "testing"))]
    pub fn new() -> Result<Self, ConfigError> {
        Self::new_internal()
    }

    fn new_internal() -> Result<Self, ConfigError> {
        let config_dir = dirs::config_local_dir().map(|dir| dir.join("rari").join("config.toml"));
        let mut s = Config::builder();
        if let Some(config_dir) = config_dir {
            s = s.add_source(File::from(config_dir).required(false));
        }
        let s = s
            .add_source(File::with_name(".config.toml").required(false))
            .add_source(
                Environment::default()
                    .list_separator(",")
                    .with_list_parse_key("additional_locales_for_generics_and_spas")
                    .with_list_parse_key("optional_translated_locales")
                    .try_parsing(true),
            )
            .build()?;

        let mut settings: Self = s.try_deserialize::<Self>()?;
        settings.validate_optional_translated_locales()?;
        let mut settings = settings.validate();
        settings.blog_root = settings
            .blog_root
            .and_then(|br| br.parent().map(|p| p.to_path_buf()));
        settings
            .build_out_root
            .get_or_insert_with(|| PathBuf::from("build"));
        Ok(settings)
    }
}

#[cfg(test)]
mod test {
    use config::FileFormat;
    use serde_json::Value;

    use super::*;

    fn embedded_package_json() -> Value {
        serde_json::from_str(PINNED_DEPS).expect("embedded deps must be valid json")
    }

    #[test]
    fn embedded_pins_are_valid() {
        let _ = Deps::new();
    }

    #[test]
    fn missing_key_is_rejected() {
        let mut json = embedded_package_json();
        let dependencies = json["dependencies"].as_object_mut().unwrap();
        let key = dependencies.keys().next().unwrap().clone();
        dependencies.remove(&key);
        assert!(serde_json::from_value::<DepsPackageJson>(json).is_err());
    }

    #[test]
    fn extra_key_is_rejected() {
        let mut json = embedded_package_json();
        json["dependencies"]["not-a-real-dependency"] = Value::from("^1.0.0");
        assert!(serde_json::from_value::<DepsPackageJson>(json).is_err());
    }

    #[test]
    fn parses_locale_source_mapping() {
        let source = r#"
            content_root = "/content/files"
            content_translated_root = "/translated-content/files"
            [translated_content_sources.de]
            root = "/translated-content-de/files"
            repository = "translated-content-de"
        "#;
        let config = Config::builder()
            .add_source(File::from_str(source, FileFormat::Toml))
            .build()
            .unwrap();
        let settings: Settings = config.try_deserialize().unwrap();
        let de = &settings.translated_content_sources[&Locale::De];
        assert_eq!(de.root, PathBuf::from("/translated-content-de/files"));
        assert_eq!(de.repository, "translated-content-de");
    }

    #[test]
    fn optional_translated_locales_parse_and_validate() {
        struct Case {
            name: &'static str,
            values: &'static [&'static str],
            expected: Option<Vec<Locale>>,
        }
        let cases = [
            Case {
                name: "default",
                values: &[],
                expected: Some(vec![]),
            },
            Case {
                name: "configured",
                values: &["de", " fr "],
                expected: Some(vec![Locale::De, Locale::Fr]),
            },
            Case {
                name: "Italian",
                values: &["it"],
                expected: Some(vec![Locale::It]),
            },
            Case {
                name: "duplicate",
                values: &["de", "de"],
                expected: Some(vec![Locale::De]),
            },
            Case {
                name: "english",
                values: &["en-US"],
                expected: None,
            },
            Case {
                name: "unsupported",
                values: &["xx"],
                expected: None,
            },
            Case {
                name: "empty",
                values: &[""],
                expected: None,
            },
        ];
        for case in cases {
            let value = serde_json::json!({ "optional_translated_locales": case.values });
            let actual = serde_json::from_value::<Settings>(value).and_then(|mut settings| {
                settings
                    .validate_optional_translated_locales()
                    .map_err(serde::de::Error::custom)?;
                Ok(settings.optional_translated_locales)
            });
            match case.expected {
                Some(expected) => assert_eq!(actual.unwrap(), expected, "{}", case.name),
                None => assert!(actual.is_err(), "{}", case.name),
            }
        }
    }

    #[test]
    fn optional_translated_locales_parse_from_environment() {
        const KEY: &str = "RARI_OPTIONAL_LOCALES_TEST_OPTIONAL_TRANSLATED_LOCALES";
        unsafe { std::env::set_var(KEY, "de, fr") };
        let config = Config::builder()
            .add_source(
                Environment::with_prefix("RARI_OPTIONAL_LOCALES_TEST")
                    .prefix_separator("_")
                    .list_separator(",")
                    .with_list_parse_key("optional_translated_locales")
                    .try_parsing(true),
            )
            .build()
            .unwrap();
        unsafe { std::env::remove_var(KEY) };
        let settings: Settings = config.try_deserialize().unwrap();
        assert_eq!(
            settings.optional_translated_locales,
            [Locale::De, Locale::Fr]
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn optional_mapped_source_may_be_absent() {
        struct Case {
            name: &'static str,
            root_exists: bool,
            optional: bool,
            expected_ok: bool,
        }
        let fixture = std::env::temp_dir().join(format!(
            "rari-optional-settings-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(fixture.join("content")).unwrap();
        let cases = [
            Case {
                name: "missing optional root",
                root_exists: false,
                optional: true,
                expected_ok: true,
            },
            Case {
                name: "missing required root",
                root_exists: false,
                optional: false,
                expected_ok: false,
            },
            Case {
                name: "missing optional directory",
                root_exists: true,
                optional: true,
                expected_ok: true,
            },
            Case {
                name: "missing required directory",
                root_exists: true,
                optional: false,
                expected_ok: false,
            },
        ];
        for (index, case) in cases.into_iter().enumerate() {
            let root = fixture.join(format!("source-{index}"));
            if case.root_exists {
                std::fs::create_dir_all(&root).unwrap();
            }
            let mut settings = Settings {
                content_root: fixture.join("content"),
                optional_translated_locales: if case.optional {
                    vec![Locale::De]
                } else {
                    vec![]
                },
                ..Settings::default()
            };
            settings.translated_content_sources.insert(
                Locale::De,
                TranslatedContentSource {
                    root,
                    repository: "translated-content-de".into(),
                },
            );
            assert_eq!(
                std::panic::catch_unwind(|| settings.validate()).is_ok(),
                case.expected_ok,
                "{}",
                case.name
            );
        }
        std::fs::remove_dir_all(fixture).unwrap();
    }
}
