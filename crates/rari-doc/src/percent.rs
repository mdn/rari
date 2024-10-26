use percent_encoding::{AsciiSet, CONTROLS};

/// A set of ASCII characters that are to be percent-encoded in URL fragments.
///
/// The `FRAGMENT` constant is an `&AsciiSet` that includes `CONTROLS` characters (The set of 0x00 to 0x1F, and 0x7F) and
/// these specific ASCII characters:
/// - Space (`' '`)
/// - Double quote (`'"'`)
/// - Less than (`'<'`)
/// - Greater than (`'>'`)
/// - Backtick (``'`'``)
///
/// For more details, see the [URL Standard](https://url.spec.whatwg.org/#fragment-percent-encode-set).
const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');

/// A set of ASCII characters that are to be percent-encoded in URL paths.
///
/// The `PATH` constant is an `&AsciiSet` that includes characters from the `FRAGMENT` set
/// and additional specific ASCII characters that are to be percent-encoded in URL paths:
/// - Hash (`'#'`)
/// - Question mark (`'?'`)
/// - Left curly brace (`'{'`)
/// - Right curly brace (`'}'`)
///
/// For more details, see the [URL Standard](https://url.spec.whatwg.org/#path-percent-encode-set).
const PATH: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');

/// A set of ASCII characters that are to be percent-encoded in individual URL userinfo components (name, password).
///
/// The `USERINFO` constant is an `&AsciiSet` that includes characters from the `PATH` set
/// and additional specific ASCII characters that are to be percent-encoded in URL userinfo components:
/// - Slash (`'/'`)
/// - Colon (`':'`)
/// - Semicolon (`';'`)
/// - Equals (`'='`)
/// - At sign (`'@'`)
/// - Left square bracket (`'['`)
/// - Backslash (`'\\'`)
/// - Right square bracket (`']'`)
/// - Caret (`'^'`)
/// - Vertical bar (`'|'`)
///
/// For more details, see the [URL Standard](https://url.spec.whatwg.org/#userinfo-percent-encode-set).
pub const USERINFO: &AsciiSet = &PATH
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

/// A set of ASCII characters that are considered to be percent-encoded in URL path segments.
///
/// The `PATH_SEGMENT` constant is an `&AsciiSet` that includes characters from the `PATH` set
/// and additional specific ASCII characters that are not allowed in URL path segments:
/// - Slash (`'/'`)
/// - Percent (`'%'`)
///
/// For more details, see the [URL Standard](https://url.spec.whatwg.org/#path-percent-encode-set).
pub const PATH_SEGMENT: &AsciiSet = &PATH.add(b'/').add(b'%');

/// A set of ASCII characters that are to be percent-encoded for use in special URL path segments.
///
/// The `SPECIAL_PATH_SEGMENT` constant is an `&AsciiSet` that includes characters from the `PATH_SEGMENT` set
/// and the backslash (`'\\'`) character. The backslash character is treated as a path separator in special URLs,
/// so it needs to be additionally escaped in that case.
pub const SPECIAL_PATH_SEGMENT: &AsciiSet = &PATH_SEGMENT.add(b'\\');

/// A set of ASCII characters that are to be percent-encoded in URL query components.
///
/// The `QUERY` constant is an `&AsciiSet` that includes `CONTROLS` characters and
/// specific ASCII characters that have to be percent-encoded in URL query components:
/// - Space (`' '`)
/// - Double quote (`'"'`)
/// - Hash (`'#'`)
/// - Less than (`'<'`)
/// - Greater than (`'>'`)
///
/// For more information, see the [URL Standard](https://url.spec.whatwg.org/#query-state).
pub const QUERY: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');

/// A set of ASCII characters that are to be percent-encoded in special URL query components.
///
/// The `SPECIAL_QUERY` constant is an `&AsciiSet` that includes characters from the `QUERY` set
/// and the single quote (`'\''`) character.
pub const SPECIAL_QUERY: &AsciiSet = &QUERY.add(b'\'');
