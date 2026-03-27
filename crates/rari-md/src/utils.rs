use std::fmt;
use std::fmt::Write;

use crate::ctype::isspace;

pub fn tagfilter(literal: &str) -> bool {
    static TAGFILTER_BLACKLIST: [&str; 9] = [
        "title",
        "textarea",
        "style",
        "xmp",
        "iframe",
        "noembed",
        "noframes",
        "script",
        "plaintext",
    ];

    let bytes = literal.as_bytes();

    if bytes.len() < 3 || bytes[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if bytes[i] == b'/' {
        i += 1;
    }

    let lc = literal[i..].to_lowercase();
    for t in TAGFILTER_BLACKLIST.iter() {
        if lc.starts_with(t) {
            let j = i + t.len();
            return isspace(bytes[j])
                || bytes[j] == b'>'
                || (bytes[j] == b'/' && bytes.len() >= j + 2 && bytes[j + 1] == b'>');
        }
    }

    false
}

pub fn tagfilter_block(input: &str, o: &mut dyn Write) -> fmt::Result {
    let bytes = input.as_bytes();
    let size = bytes.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && bytes[i] != b'<' {
            i += 1;
        }

        if i > org {
            o.write_str(&input[org..i])?;
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            o.write_str("&lt;")?;
        } else {
            o.write_str("<")?;
        }

        i += 1;
    }

    Ok(())
}
pub fn escape_href(output: &mut dyn Write, buffer: &str) -> fmt::Result {
    let bytes = buffer.as_bytes();
    let size = bytes.len();
    let mut i = 0;
    let mut escaped = "";

    while i < size {
        let org = i;
        while i < size {
            escaped = match bytes[i] {
                b'&' => "&amp;",
                b'<' => "&lt;",
                b'>' => "&gt;",
                b'"' => "&quot;",
                b'\'' => "&#x27;",
                _ => {
                    i += 1;
                    ""
                }
            };
            if !escaped.is_empty() {
                break;
            }
        }

        if i > org {
            output.write_str(&buffer[org..i])?;
        }

        if !escaped.is_empty() {
            output.write_str(escaped)?;
            escaped = "";
            i += 1;
        }
    }

    Ok(())
}
