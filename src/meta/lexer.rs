use crate::token::{Token, TokenType, Position};

pub struct Lexer {
    source: Vec<char>,
    text: String,
    tokens: Vec<Token>,
    cursor: usize,
    row: usize,
    line_start: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source: source.clone().chars().collect(),
            text: source,
            tokens: Vec::new(),
            cursor: 0,
            row: 0,
            line_start: 0,
        }
    }

    pub fn make_tokens(&mut self) {
        while self.valid() {
            let first = self.character();
            let pos = Position::from("Script.mt".to_string(), self.row as u32, (self.cursor - self.line_start) as u32);

            if first.is_alphanumeric() {
                self.parse_alnum_token(pos);
            } else if first.is_ascii_digit() {
                self.parse_digit_token();
            } else if first.is_whitespace() {
                self.cursor += 1;
            }
        }
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    pub fn valid(&self) -> bool {
        self.cursor < self.source.len()
    }

    fn character(&self) -> char {
        self.source[self.cursor]
    }

    fn parse_alnum_token(&mut self, pos: Position) {
        let start = self.cursor;
        while self.valid() && self.character().is_alphanumeric() {
            self.cursor += 1;
        }
        
        let value = String::from(&self.text[start..self.cursor]);
        self.tokens.push(Token::from(TokenType::Ident, value, pos));
    }

    fn parse_digit_token(&mut self) {
        let first = self.source[self.cursor];
    }
}
