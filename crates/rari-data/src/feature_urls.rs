use indexmap::IndexMap;
use std::sync::LazyLock;

static FEATURE_MDN_URLS: LazyLock<IndexMap<&'static str, &'static str>> = LazyLock::new(|| {
    [
        ("async-clipboard", "/docs/Web/API/Clipboard_API"),
        ("background-color", "/docs/Web/CSS/background-color"),
        ("clip-path", "/docs/Web/CSS/clip-path"),
        ("color", "/docs/Web/CSS/color"),
        ("container-style-queries", "/docs/Web/CSS/@container"),
        (
            "contenteditable",
            "/docs/Web/HTML/Reference/Global_attributes/contenteditable",
        ),
        ("css-object-model", "/docs/Web/API/CSS_Object_Model"),
        (
            "destructuring",
            "/docs/Web/JavaScript/Reference/Operators/Destructuring",
        ),
        ("dom", "/docs/Web/API/Document_Object_Model"),
        ("font-face", "/docs/Web/CSS/@font-face"),
        (
            "get-computed-style",
            "/docs/Web/API/Window/getComputedStyle",
        ),
        ("media-queries", "/docs/Web/CSS/@media"),
        ("mutationobserver", "/docs/Web/API/MutationObserver"),
        (
            "navigation-timing",
            "/docs/Web/API/Performance_API/Navigation_timing",
        ),
        ("page-breaks", "/docs/Web/CSS/break-after"),
        ("performance", "/docs/Web/API/Performance_API"),
        ("scheduler", "/docs/Web/API/Prioritized_Task_Scheduling_API"),
        ("svg", "/docs/Web/SVG"),
        ("webxr-device", "/docs/Web/API/WebXR_Device_API"),
    ]
    .into_iter()
    .collect()
});

pub fn get_mdn_url(feature_key: &str) -> Option<&'static str> {
    FEATURE_MDN_URLS.get(feature_key).copied()
}
