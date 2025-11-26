//! Utilities for converting between byte offsets and character positions.
//!
//! This module provides functions to convert between:
//! - Byte offsets (used internally for string slicing and tree-sitter)
//! - Character positions (used for user-facing display and LSP)
//!
//! ## Background
//!
//! In Rust, strings are UTF-8 encoded byte sequences. A single Unicode character
//! can be 1-4 bytes. This creates two different ways to measure positions:
//!
//! 1. **Byte offset**: The number of bytes from the start (used for string slicing)
//! 2. **Character position**: The number of Unicode characters from the start (user-facing)
//!
//! For example, in "Hello 🔥 world":
//! - The 🔥 emoji is at byte offset 6 but character position 6
//! - The emoji itself is 4 bytes long but counts as 1 character
//! - The space after it is at byte offset 10 but character position 7

/// Convert a byte offset within a line to a character position.
///
/// # Arguments
///
/// * `line` - The line content
/// * `byte_offset` - The byte offset within the line (0-based)
///
/// # Returns
///
/// The character position (0-based) corresponding to the byte offset.
/// If the byte offset is beyond the line length, returns the character count of the line.
///
/// # Examples
///
/// ```
/// use rari_doc::position_utils::byte_to_char_column;
///
/// let line = "Hello 🔥 world";
/// assert_eq!(byte_to_char_column(line, 0), 0);  // 'H'
/// assert_eq!(byte_to_char_column(line, 6), 6);  // '🔥' starts at byte 6, char 6
/// assert_eq!(byte_to_char_column(line, 10), 7); // space after emoji at byte 10, char 7
/// ```
pub fn byte_to_char_column(line: &str, byte_offset: usize) -> usize {
    if byte_offset >= line.len() {
        return line.chars().count();
    }

    line[..byte_offset].chars().count()
}

/// Convert a character position within a line to a byte offset.
///
/// # Arguments
///
/// * `line` - The line content
/// * `char_offset` - The character position within the line (0-based)
///
/// # Returns
///
/// The byte offset (0-based) corresponding to the character position.
/// If the character position is beyond the line length, returns the byte length of the line.
///
/// # Examples
///
/// ```
/// use rari_doc::position_utils::char_to_byte_column;
///
/// let line = "Hello 🔥 world";
/// assert_eq!(char_to_byte_column(line, 0), 0);  // 'H'
/// assert_eq!(char_to_byte_column(line, 6), 6);  // '🔥' at char 6, byte 6
/// assert_eq!(char_to_byte_column(line, 7), 10); // space after emoji at char 7, byte 10
/// ```
pub fn char_to_byte_column(line: &str, char_offset: usize) -> usize {
    let mut byte_pos = 0;
    let mut char_count = 0;

    for ch in line.chars() {
        if char_count >= char_offset {
            return byte_pos;
        }
        byte_pos += ch.len_utf8();
        char_count += 1;
    }

    byte_pos
}

/// Get byte offset from line and character column position in content.
///
/// # Arguments
///
/// * `content` - The full content
/// * `line` - The line number (0-based)
/// * `char_col` - The character column within the line (0-based)
///
/// # Returns
///
/// The byte offset from the start of content, or `None` if the line doesn't exist.
pub fn line_char_col_to_byte_offset(content: &str, line: usize, char_col: usize) -> Option<usize> {
    let line_content = content.lines().nth(line)?;
    let byte_col = char_to_byte_column(line_content, char_col);

    // Calculate byte offset from start of content
    let bytes_before_line: usize = content
        .lines()
        .take(line)
        .map(|l| l.len() + 1) // +1 for newline
        .sum();

    Some(bytes_before_line + byte_col)
}

/// Get byte offset from line and byte column position in content.
///
/// # Arguments
///
/// * `content` - The full content
/// * `line` - The line number (0-based)
/// * `byte_col` - The byte column within the line (0-based)
///
/// # Returns
///
/// The byte offset from the start of content, or `None` if the line doesn't exist.
pub fn line_byte_col_to_byte_offset(content: &str, line: usize, byte_col: usize) -> Option<usize> {
    let line_content = content.lines().nth(line)?;

    // Ensure byte_col is within bounds and on a char boundary
    let byte_col = byte_col.min(line_content.len());
    if !line_content.is_char_boundary(byte_col) {
        return None;
    }

    // Calculate byte offset from start of content
    let bytes_before_line: usize = content
        .lines()
        .take(line)
        .map(|l| l.len() + 1) // +1 for newline
        .sum();

    Some(bytes_before_line + byte_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_char_ascii() {
        let line = "Hello world";
        assert_eq!(byte_to_char_column(line, 0), 0);
        assert_eq!(byte_to_char_column(line, 5), 5);
        assert_eq!(byte_to_char_column(line, 11), 11);
    }

    #[test]
    fn test_byte_to_char_emoji() {
        let line = "Hello 🔥 world";
        // "Hello " = 6 bytes, 6 chars
        // "🔥" = 4 bytes, 1 char (at byte 6, char 6)
        // " world" starts at byte 10, char 7
        assert_eq!(byte_to_char_column(line, 0), 0);
        assert_eq!(byte_to_char_column(line, 6), 6); // Start of emoji
        assert_eq!(byte_to_char_column(line, 10), 7); // After emoji
        assert_eq!(byte_to_char_column(line, 11), 8); // 'w'
    }

    #[test]
    fn test_byte_to_char_accented() {
        let line = "Café"; // é is 2 bytes
        assert_eq!(byte_to_char_column(line, 0), 0); // 'C'
        assert_eq!(byte_to_char_column(line, 3), 3); // 'é' starts at byte 3
        assert_eq!(byte_to_char_column(line, 5), 4); // After 'é'
    }

    #[test]
    fn test_byte_to_char_beyond_end() {
        let line = "Hello";
        assert_eq!(byte_to_char_column(line, 100), 5);
    }

    #[test]
    fn test_char_to_byte_ascii() {
        let line = "Hello world";
        assert_eq!(char_to_byte_column(line, 0), 0);
        assert_eq!(char_to_byte_column(line, 5), 5);
        assert_eq!(char_to_byte_column(line, 11), 11);
    }

    #[test]
    fn test_char_to_byte_emoji() {
        let line = "Hello 🔥 world";
        assert_eq!(char_to_byte_column(line, 0), 0); // 'H'
        assert_eq!(char_to_byte_column(line, 6), 6); // '🔥'
        assert_eq!(char_to_byte_column(line, 7), 10); // ' ' after emoji
        assert_eq!(char_to_byte_column(line, 8), 11); // 'w'
    }

    #[test]
    fn test_char_to_byte_accented() {
        let line = "Café";
        assert_eq!(char_to_byte_column(line, 0), 0); // 'C'
        assert_eq!(char_to_byte_column(line, 3), 3); // 'é'
        assert_eq!(char_to_byte_column(line, 4), 5); // After string
    }

    #[test]
    fn test_char_to_byte_beyond_end() {
        let line = "Hello";
        assert_eq!(char_to_byte_column(line, 100), 5);
    }

    #[test]
    fn test_roundtrip_ascii() {
        let line = "Hello world";
        for i in 0..=line.len() {
            let char_pos = byte_to_char_column(line, i);
            let byte_pos = char_to_byte_column(line, char_pos);
            assert_eq!(byte_pos, i);
        }
    }

    #[test]
    fn test_roundtrip_emoji() {
        let line = "Hi 🔥!";
        // Test at character boundaries
        assert_eq!(char_to_byte_column(line, byte_to_char_column(line, 0)), 0);
        assert_eq!(char_to_byte_column(line, byte_to_char_column(line, 3)), 3);
        assert_eq!(char_to_byte_column(line, byte_to_char_column(line, 7)), 7);
    }

    #[test]
    fn test_line_char_col_to_byte_offset() {
        let content = "Hello\n🔥 world\nGoodbye";

        // Line 0, char 0: 'H' at byte 0
        assert_eq!(line_char_col_to_byte_offset(content, 0, 0), Some(0));

        // Line 0, char 5: end of "Hello" at byte 5
        assert_eq!(line_char_col_to_byte_offset(content, 0, 5), Some(5));

        // Line 1, char 0: '🔥' at byte 6 (after "Hello\n")
        assert_eq!(line_char_col_to_byte_offset(content, 1, 0), Some(6));

        // Line 1, char 2: 'w' at byte 12 (6 + 4 bytes for emoji + 1 for space + 1 for 'w' start)
        assert_eq!(line_char_col_to_byte_offset(content, 1, 2), Some(11));

        // Line 2, char 0: 'G' at byte 17 (5 bytes line0 + 1 newline + 10 bytes line1 + 1 newline)
        assert_eq!(line_char_col_to_byte_offset(content, 2, 0), Some(17));

        // Invalid line
        assert_eq!(line_char_col_to_byte_offset(content, 10, 0), None);
    }

    #[test]
    fn test_line_byte_col_to_byte_offset() {
        let content = "Hello\n🔥 world\nGoodbye";

        // Line 0, byte 0
        assert_eq!(line_byte_col_to_byte_offset(content, 0, 0), Some(0));

        // Line 1, byte 0: emoji starts at byte 6
        assert_eq!(line_byte_col_to_byte_offset(content, 1, 0), Some(6));

        // Line 1, byte 5: space after emoji at byte 11
        assert_eq!(line_byte_col_to_byte_offset(content, 1, 5), Some(11));

        // Invalid line
        assert_eq!(line_byte_col_to_byte_offset(content, 10, 0), None);
    }

    #[test]
    fn test_multiple_emojis() {
        let line = "Test 🔥🌟✨ end";
        // "Test " = 5 bytes, 5 chars
        // "🔥" = 4 bytes, 1 char (byte 5, char 5)
        // "🌟" = 4 bytes, 1 char (byte 9, char 6)
        // "✨" = 3 bytes, 1 char (byte 13, char 7)
        // " end" starts at byte 16, char 8

        assert_eq!(byte_to_char_column(line, 5), 5); // Start of 🔥
        assert_eq!(byte_to_char_column(line, 9), 6); // Start of 🌟
        assert_eq!(byte_to_char_column(line, 13), 7); // Start of ✨
        assert_eq!(byte_to_char_column(line, 16), 8); // Space after emojis

        assert_eq!(char_to_byte_column(line, 5), 5); // 🔥
        assert_eq!(char_to_byte_column(line, 6), 9); // 🌟
        assert_eq!(char_to_byte_column(line, 7), 13); // ✨
        assert_eq!(char_to_byte_column(line, 8), 16); // Space
    }
}
