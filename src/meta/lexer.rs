use crate::token::{LiteralType, Position, Token, TokenType};

pub struct Lexer {
    chars: Vec<char>,
    filename: String,
    source: String,
    cursor: usize,
    row: usize,
    line_start: usize,
}

impl Lexer {
    pub fn new(source: String, filename: String) -> Self {
        Self {
            chars: source.clone().chars().collect(),
            filename,
            source,
            cursor: 0,
            row: 0,
            line_start: 0,
        }
    }

    pub fn advance(&mut self) {
        self.cursor += 1;
    }

    pub fn valid(&self) -> bool {
        self.cursor < self.chars.len()
    }

    pub fn character(&self) -> char {
        self.chars[self.cursor]
    }

    pub fn peek_char(&self) -> Option<char> {
        if self.cursor + 1 < self.chars.len() {
            return Some(self.chars[self.cursor + 1]);
        }

        None
    }

    pub fn get_cursor_pos(&self) -> Position {
        Position::from(
            self.filename.clone(),
            self.row as u32,
            (self.cursor - self.line_start) as u32,
        )
    }

    pub fn trim(&mut self) {
        let mut c = self.character();
        while self.valid() && c.is_ascii_whitespace() {
            self.advance();

            if c == '\n' {
                self.row += 1;
                self.line_start = self.cursor;
            }

            if !self.valid() {
                break;
            }

            c = self.character();
        }
    }

    fn drop_line(&mut self) {
        while self.valid() && self.character() != '\n' {
            self.advance();
        }

        self.advance();
    }

    fn parse_string(&mut self, pos: Position) -> Option<Token> {
        self.advance();

        let start = self.cursor;

        let mut c = self.character();
        while self.valid()
            && c != '"'
            && (c.is_alphanumeric() || c.is_ascii_whitespace() || c.is_ascii_punctuation())
        {
            self.advance();
            c = self.character();
        }

        let value = String::from(&self.source[start..self.cursor]);
        let token = Some(Token::from(
            TokenType::Literal(LiteralType::String),
            value,
            pos,
        ));

        self.advance();

        token
    }

    fn parse_punctuation_token(&mut self, pos: Position) -> Option<Token> {
        let token = self.character();

        self.advance();

        match token {
            '(' => Some(Token::from(TokenType::Oparen, String::from(token), pos)),
            ')' => Some(Token::from(TokenType::Cparen, String::from(token), pos)),
            '{' => Some(Token::from(TokenType::Ocurly, String::from(token), pos)),
            '}' => Some(Token::from(TokenType::Ccurly, String::from(token), pos)),
            ':' => Some(Token::from(TokenType::Colon, String::from(token), pos)),
            ';' => Some(Token::from(TokenType::Semicolon, String::from(token), pos)),
            ',' => Some(Token::from(TokenType::Comma, String::from(token), pos)),
            _ => None,
        }
    }

    fn parse_operator_token(&mut self, pos: Position) -> Option<Token> {
        let op = self.character();

        self.advance();

        match op {
            '+' => Some(Token::from(TokenType::Plus, String::from(op), pos)),
            '-' => Some(Token::from(TokenType::Minus, String::from(op), pos)),
            '*' => Some(Token::from(TokenType::Multiply, String::from(op), pos)),
            '/' => Some(Token::from(TokenType::Divide, String::from(op), pos)),
            '=' => Some(Token::from(TokenType::Equal, String::from(op), pos)),
            _ => None,
        }
    }

    fn parse_ident_token(&mut self, pos: Position) -> Option<Token> {
        let start = self.cursor;
        let mut c = self.character();
        while self.valid() && c.is_alphanumeric() || c == '_' {
            self.advance();
            c = self.character();
        }

        let value = String::from(&self.source[start..self.cursor]);

        let token_type = match value.as_str() {
            "proc" => TokenType::Proc,
            "let" => TokenType::Let,
            "return" => TokenType::Return,
            _ => TokenType::Ident,
        };

        Some(Token::from(token_type, value, pos))
    }

    fn parse_digit_token(&mut self, pos: Position) -> Option<Token> {
        let start = self.cursor;
        let mut c = self.character();
        while self.valid() && c.is_ascii_digit() {
            self.advance();
            c = self.character();
        }

        let value = String::from(&self.source[start..self.cursor]);
        Some(Token::from(
            TokenType::Literal(LiteralType::Number),
            value,
            pos,
        ))
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.valid() {
            return None;
        }

        if self.character().is_ascii_whitespace() {
            self.trim();

            if !self.valid() {
                return None;
            }
        }

        if self.character() == '/' {
            if let Some(c) = self.peek_char() {
                if c == '/' {
                    self.drop_line();
                }
            }
        }

        let first = self.character();
        let pos = self.get_cursor_pos();

        let punctuation_tokens = "(){};:,";
        let operator_tokens = "+-*/=";

        if first == '"' {
            self.parse_string(pos)
        } else if punctuation_tokens.contains(first) {
            self.parse_punctuation_token(pos)
        } else if operator_tokens.contains(first) {
            self.parse_operator_token(pos)
        } else if first.is_ascii_alphabetic() || first == '_' {
            self.parse_ident_token(pos)
        } else if first.is_ascii_digit() {
            self.parse_digit_token(pos)
        } else {
            None
        }
    }
}
