use once_cell::sync::Lazy;
use regex::Regex;

pub fn anchorize(content: &str) -> String {
    static REJECTED_CHARS: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"[<>"$#%&+,/:;=?@\[\]^`{|}~')(\\]"#).unwrap());

    let mut id = content.to_lowercase();
    id = REJECTED_CHARS.replace_all(&id, "").replace(' ', "_");
    id
}
