use crate::error::SyntaxDefinitionError;

pub struct Tokenizer {
    pub str: Vec<char>,
    pub pos: usize,
}

impl Tokenizer {
    pub fn new(str: &str) -> Tokenizer {
        Tokenizer {
            str: str.chars().collect(),
            pos: 0,
        }
    }

    pub fn char_code_at(&self, pos: usize) -> char {
        if pos < self.str.len() {
            self.str[pos]
        } else {
            '\0'
        }
    }

    pub fn char_code(&self) -> char {
        self.char_code_at(self.pos)
    }

    pub fn next_char_code(&self) -> char {
        self.char_code_at(self.pos + 1)
    }

    pub fn next_non_ws_code(&self, pos: usize) -> char {
        self.char_code_at(self.find_ws_end(pos))
    }

    pub fn find_ws_end(&self, pos: usize) -> usize {
        self.str
            .iter()
            .skip(pos)
            .position(|c| !matches!(c, '\r' | '\n' | '\u{c}' | ' ' | '\t'))
            .map(|p| pos + p)
            .unwrap_or(self.str.len())
    }

    pub fn substring_to_pos(&mut self, end: usize) -> String {
        let substring = self
            .str
            .iter()
            .skip(self.pos)
            .take(end - self.pos)
            .collect();
        self.pos = end;
        substring
    }

    pub fn eat(&mut self, code: char) -> Result<(), SyntaxDefinitionError> {
        if self.char_code() != code {
            return Err(SyntaxDefinitionError::ParseErrorExpected(code));
        }

        self.pos += 1;
        Ok(())
    }

    pub fn peek(&mut self) -> char {
        if self.pos < self.str.len() {
            let ch = self.str[self.pos];
            self.pos += 1;
            ch
        } else {
            '\0'
        }
    }

    pub fn error(&self, message: &str) {
        eprintln!("Tokenizer error: {message}");
        panic!("Tokenizer error: {message}");
    }
}
