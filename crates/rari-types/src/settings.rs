use std::collections::BTreeMap;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

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

impl Settings {
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
            source.root = std::fs::canonicalize(&source.root).unwrap_or_else(|_| {
                panic!("translated content root for {locale} is not a valid path")
            });
            if self.content_translated_root.as_ref() == Some(&source.root) {
                assert_eq!(
                    source.repository, "translated-content",
                    "repository for {locale} conflicts with CONTENT_TRANSLATED_ROOT"
                );
            }
            assert!(
                source.root.join(locale.as_folder_str()).is_dir(),
                "translated content root for {locale} has no locale directory"
            );
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
                    .try_parsing(true),
            )
            .build()?;

        let mut settings: Self = s.try_deserialize::<Self>()?.validate();
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
}
