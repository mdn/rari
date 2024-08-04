use std::sync::LazyLock;

use rari_data::baseline::{SupportStatus, WebFeatures};
use rari_types::globals::data_dir;
use tracing::warn;

static WEB_FEATURES: LazyLock<Option<WebFeatures>> = LazyLock::new(|| {
    let web_features = WebFeatures::from_file(
        &data_dir()
            .join("web-features")
            .join("package")
            .join("data.json"),
    );
    match web_features {
        Ok(web_features) => Some(web_features),
        Err(e) => {
            warn!("{e:?}");
            None
        }
    }
});

static DISALLOW_LIST: &[&str] = &[
    // https://github.com/web-platform-dx/web-features/blob/cf718ad/feature-group-definitions/async-clipboard.yml
    "api.Clipboard.read",
    "api.Clipboard.readText",
    "api.Clipboard.write",
    "api.Clipboard.writeText",
    "api.ClipboardEvent",
    "api.ClipboardEvent.ClipboardEvent",
    "api.ClipboardEvent.clipboardData",
    "api.ClipboardItem",
    "api.ClipboardItem.ClipboardItem",
    "api.ClipboardItem.getType",
    "api.ClipboardItem.presentationStyle",
    "api.ClipboardItem.types",
    "api.Navigator.clipboard",
    "api.Permissions.permission_clipboard-read",
    // https://github.com/web-platform-dx/web-features/blob/cf718ad/feature-group-definitions/custom-elements.yml
    "api.CustomElementRegistry",
    "api.CustomElementRegistry.builtin_element_support",
    "api.CustomElementRegistry.define",
    "api.Window.customElements",
    "css.selectors.defined",
    "css.selectors.host",
    "css.selectors.host-context",
    "css.selectors.part",
    // https://github.com/web-platform-dx/web-features/blob/cf718ad/feature-group-definitions/input-event.yml
    "api.Element.input_event",
    "api.InputEvent.InputEvent",
    "api.InputEvent.data",
    "api.InputEvent.dataTransfer",
    "api.InputEvent.getTargetRanges",
    "api.InputEvent.inputType",
    // https://github.com/web-platform-dx/web-features/issues/1038
    // https://github.com/web-platform-dx/web-features/blob/64d2cfd/features/screen-orientation-lock.dist.yml
    "api.ScreenOrientation.lock",
    "api.ScreenOrientation.unlock",
];

pub fn get_baseline(browser_compat: &[String]) -> Option<&'static SupportStatus> {
    if let Some(ref web_features) = *WEB_FEATURES {
        if browser_compat.is_empty() {
            return None;
        }
        let filtered_browser_compat = browser_compat.iter().filter_map(
      |query|
        // temporary blocklist while we wait for per-key baseline statuses
        // or another solution to the baseline/bcd table discrepancy problem
      if !DISALLOW_LIST.contains(&query.as_str()) {
        Some(query.as_str())
      } else {None}
    ).collect::<Vec<&str>>();
        return web_features.feature_status(&filtered_browser_compat);
    }
    None
}
