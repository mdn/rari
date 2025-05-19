use std::fs;
use std::path::{Path, PathBuf};

use config::{Config, ConfigError, Environment, File};
use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::locale::Locale;

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Deps {
    #[serde(alias = "@mdn/browser-compat-data")]
    pub bcd: Option<VersionReq>,
    #[serde(alias = "mdn-data")]
    pub mdn_data: Option<VersionReq>,
    #[serde(alias = "web-features")]
    pub web_features: Option<VersionReq>,
    #[serde(alias = "web-specs")]
    pub web_specs: Option<VersionReq>,
    #[serde(alias = "@webref/css")]
    pub webref_css: Option<VersionReq>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct DepsPackageJson {
    dependencies: Deps,
}

impl Deps {
    pub fn new() -> Result<Self, ConfigError> {
        if let Some(package_json) =
            std::env::var_os("DEPS_PACKAGE_JSON").or_else(|| std::env::var_os("deps_package_json"))
        {
            let path = Path::new(&package_json);
            if let Some(deps_json) = fs::read_to_string(path).ok().and_then(|json_str| {
                let s = serde_json::from_str::<DepsPackageJson>(&json_str);
                s.ok()
            }) {
                return Ok(deps_json.dependencies);
            } else {
                tracing::error!("unable to parse {}", path.display());
            }
        }
        let s = Config::builder()
            .add_source(Environment::default().prefix("deps").try_parsing(true))
            .build()?;

        let deps: Self = s.try_deserialize::<Self>()?;
        Ok(deps)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(default)]
pub struct Settings {
    pub content_root: PathBuf,
    pub content_translated_root: Option<PathBuf>,
    pub build_out_root: Option<PathBuf>,
    pub blog_root: Option<PathBuf>,
    pub generic_content_root: Option<PathBuf>,
    pub curriculum_root: Option<PathBuf>,
    pub contributor_spotlight_root: Option<PathBuf>,
    pub deny_warnings: bool,
    pub cache_content: bool,
    pub base_url: String,
    pub live_samples_base_url: String,
    pub legacy_live_samples_base_url: String,
    pub interactive_examples_base_url: String,
    pub additional_locales_for_generics_and_spas: Vec<Locale>,
    pub reader_ignores_gitignore: bool,
    pub data_issues: bool,
    pub json_issues: bool,
    pub json_live_samples: bool,
    pub blog_unpublished: bool,
    pub deps: Deps,
    pub blog_pagination: bool,
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
        self
    }

    #[cfg(target_arch = "wasm32")]
    fn validate(self) -> Self {
        self
    }

    #[cfg(feature = "testing")]
    pub fn new() -> Result<Self, ConfigError> {
        std::env::set_var(
            "CONTENT_ROOT",
            std::env::var("TESTING_CONTENT_ROOT").unwrap(),
        );
        std::env::set_var(
            "CONTENT_TRANSLATED_ROOT",
            std::env::var("TESTING_CONTENT_TRANSLATED_ROOT").unwrap(),
        );
        std::env::set_var(
            "CACHE_CONTENT",
            std::env::var("TESTING_CACHE_CONTENT").unwrap(),
        );
        std::env::set_var(
            "READER_IGNORES_GITIGNORE",
            std::env::var("TESTING_READER_IGNORES_GITIGNORE").unwrap(),
        );
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
        Ok(settings)
    }
}
