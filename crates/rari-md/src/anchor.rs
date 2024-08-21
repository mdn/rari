use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

pub fn anchorize(content: &str) -> Cow<'_, str> {
    static REJECTED_CHARS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"[<>"$#%&+,/:;=?@\[\]^`{|}~')(\\]"#).unwrap());

    let id = REJECTED_CHARS.replace_all(content, "");
    let mut id = id.trim().to_lowercase();
    let mut prev = ' ';
    id.retain(|c| {
        let result = !c.is_ascii_whitespace() || !prev.is_ascii_whitespace();
        prev = c;
        result
    });
    let id = id.replace(' ', "_");
    if !id.is_empty() {
        if id == content {
            Cow::Borrowed(content)
        } else {
            Cow::Owned(id)
        }
    } else {
        Cow::Borrowed("sect")
    }
}
