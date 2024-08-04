use std::sync::LazyLock;

use regex::Regex;

pub fn anchorize(content: &str) -> String {
    static REJECTED_CHARS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[<>"$#%&+,/:;=?@\[\]^`{|}~')(\\]"#).unwrap());

    let id = content.to_lowercase().replace(' ', "_");
    let id = REJECTED_CHARS.replace_all(&id, "");
    if !id.is_empty() {
        id.into_owned()
    } else {
        "sect1".to_string()
    }
}
