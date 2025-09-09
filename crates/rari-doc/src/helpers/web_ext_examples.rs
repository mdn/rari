use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use rari_types::globals::data_dir;
use rari_utils::io::read_to_string;
use serde::Deserialize;
use tracing::warn;

use crate::error::DocError;

#[derive(Deserialize, Debug, Clone)]
pub struct WebExtExample {
    pub description: String,
    pub javascript_apis: Vec<String>,
    pub name: String,
}

pub struct WebExtExamplesData {
    pub by_module: HashMap<&'static str, Vec<&'static WebExtExample>>,
    pub by_module_and_api: HashMap<&'static str, Vec<&'static WebExtExample>>,
}

pub static WEB_EXT_EXAMPLES_JSON: LazyLock<Vec<WebExtExample>> = LazyLock::new(|| {
    match read_to_string(data_dir().join("web_ext_examples/data.json"))
        .map_err(DocError::from)
        .and_then(|s| serde_json::from_str(&s).map_err(DocError::from))
    {
        Ok(data) => data,
        Err(e) => {
            warn!("Error loading mdn/data: {e}");
            Default::default()
        }
    }
});

pub fn web_ext_examples_json() -> &'static [WebExtExample] {
    &WEB_EXT_EXAMPLES_JSON
}

pub static WEB_EXT_EXAMPLES_DATA: LazyLock<WebExtExamplesData> = LazyLock::new(|| {
    let mut by_module = HashMap::new();
    for example in web_ext_examples_json() {
        for js_api in &example
            .javascript_apis
            .iter()
            .map(|js_api| &js_api[..js_api.find('.').unwrap_or(js_api.len())])
            .collect::<HashSet<_>>()
        {
            by_module
                .entry(*js_api)
                .and_modify(|e: &mut Vec<_>| e.push(example))
                .or_insert(vec![example]);
        }
    }
    let mut by_module_and_api = HashMap::new();
    for example in web_ext_examples_json() {
        for js_api in &example.javascript_apis {
            by_module_and_api
                .entry(js_api.as_str())
                .and_modify(|e: &mut Vec<_>| e.push(example))
                .or_insert(vec![example]);
        }
    }
    WebExtExamplesData {
        by_module,
        by_module_and_api,
    }
});
