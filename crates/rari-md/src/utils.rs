use std::fmt;
use std::fmt::Write;

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
