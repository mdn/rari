use std::io;
use std::io::Write;

use crate::ctype::isspace;

pub fn tagfilter(literal: &[u8]) -> bool {
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

    if literal.len() < 3 || literal[0] != b'<' {
        return false;
    }

    let mut i = 1;
    if literal[i] == b'/' {
        i += 1;
    }

    let lc = unsafe { String::from_utf8_unchecked(literal[i..].to_vec()) }.to_lowercase();
    for t in TAGFILTER_BLACKLIST.iter() {
        if lc.starts_with(t) {
            let j = i + t.len();
            return isspace(literal[j])
                || literal[j] == b'>'
                || (literal[j] == b'/' && literal.len() >= j + 2 && literal[j + 1] == b'>');
        }
    }

    false
}

pub fn tagfilter_block(input: &[u8], o: &mut dyn Write) -> io::Result<()> {
    let size = input.len();
    let mut i = 0;

    while i < size {
        let org = i;
        while i < size && input[i] != b'<' {
            i += 1;
        }

        if i > org {
            o.write_all(&input[org..i])?;
        }

        if i >= size {
            break;
        }

        if tagfilter(&input[i..]) {
            o.write_all(b"&lt;")?;
        } else {
            o.write_all(b"<")?;
        }

        i += 1;
    }

    Ok(())
}
pub fn escape_href(output: &mut dyn Write, buffer: &[u8]) -> io::Result<()> {
    let size = buffer.len();
    let mut i = 0;
    let mut escaped = "";

    while i < size {
        let org = i;
        while i < size {
            escaped = match buffer[i] {
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
            output.write_all(&buffer[org..i])?;
        }

        if !escaped.is_empty() {
            output.write_all(escaped.as_bytes())?;
            escaped = "";
            i += 1;
        }
    }

    Ok(())
}
