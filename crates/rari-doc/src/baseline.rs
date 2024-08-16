use std::sync::LazyLock;

use rari_data::baseline::{SupportStatusWithByKey, WebFeatures};
use rari_types::globals::data_dir;
use tracing::warn;

static WEB_FEATURES: LazyLock<Option<WebFeatures>> = LazyLock::new(|| {
    let web_features =
        WebFeatures::from_file(&data_dir().join("baseline").join("data.extended.json"));
    match web_features {
        Ok(web_features) => Some(web_features),
        Err(e) => {
            warn!("{e:?}");
            None
        }
    }
});

pub fn get_baseline(browser_compat: &[String]) -> Option<&'static SupportStatusWithByKey> {
    if let Some(ref web_features) = *WEB_FEATURES {
        return match &browser_compat {
            &[bcd_key] => web_features.feature_status(bcd_key.as_str()),
            _ => None,
        };
    }
    None
}
