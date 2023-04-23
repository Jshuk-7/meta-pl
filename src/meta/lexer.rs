use crate::token::{LiteralType, Position, Token, TokenType};

pub struct Lexer {
    filename: String,
    chars: Vec<char>,
    source: String,
    cursor: usize,
    row: usize,
    line_start: usize,
}

impl Lexer {
    pub fn new(source: String, filename: String) -> Self {
        Self {
            filename,
            chars: source.clone().chars().collect(),
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
        self.peek_char_by_amount(1)
    }

    pub fn peek_char_by_amount(&self, amount: usize) -> Option<char> {
        if self.cursor + amount < self.chars.len() {
            return Some(self.chars[self.cursor + amount]);
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

    fn parse_string_token(&mut self, pos: Position) -> Option<Token> {
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

    fn parse_char_token(&mut self, pos: Position) -> Option<Token> {
        self.advance();

        let c = self.character();

        let token = Some(Token::from(
            TokenType::Literal(LiteralType::Char),
            String::from(c),
            pos,
        ));

        self.advance();

        self.advance();

        token
    }

    fn parse_punctuation_token(&mut self, pos: Position) -> Option<Token> {
        let token = self.character();

        self.advance();

        let value = String::from(token);

        match token {
            '(' => Some(Token::from(TokenType::Oparen, value, pos)),
            ')' => Some(Token::from(TokenType::Cparen, value, pos)),
            '{' => Some(Token::from(TokenType::Ocurly, value, pos)),
            '}' => Some(Token::from(TokenType::Ccurly, value, pos)),
            ':' => Some(Token::from(TokenType::Colon, value, pos)),
            ';' => Some(Token::from(TokenType::Semicolon, value, pos)),
            ',' => Some(Token::from(TokenType::Comma, value, pos)),
            '.' => Some(Token::from(TokenType::Period, value, pos)),
            _ => None,
        }
    }

    fn parse_operator_token(&mut self, pos: Position) -> Option<Token> {
        let op = self.character();

        let next = if let Some(c) = self.peek_char() {
            c
        } else {
            0 as char
        };

        self.advance();

        match op {
            '+' => Some(Token::from(TokenType::Add, String::from(op), pos)),
            '-' => Some(Token::from(TokenType::Sub, String::from(op), pos)),
            '*' => Some(Token::from(TokenType::Mul, String::from(op), pos)),
            '/' => Some(Token::from(TokenType::Div, String::from(op), pos)),
            '=' => {
                if next == '=' {
                    self.advance();
                    Some(Token::from(TokenType::Eq, String::from("=="), pos))
                } else {
                    Some(Token::from(TokenType::Assign, String::from(op), pos))
                }
            }
            '<' => {
                if next == '=' {
                    self.advance();
                    Some(Token::from(TokenType::Lte, String::from("<="), pos))
                } else {
                    Some(Token::from(TokenType::Lt, String::from(op), pos))
                }
            }
            '>' => {
                if next == '=' {
                    self.advance();
                    Some(Token::from(TokenType::Gte, String::from(">="), pos))
                } else {
                    Some(Token::from(TokenType::Gt, String::from(op), pos))
                }
            }
            '!' => {
                if next == '=' {
                    self.advance();
                    Some(Token::from(TokenType::Ne, String::from("!="), pos))
                } else {
                    Some(Token::from(TokenType::Neg, String::from(op), pos))
                }
            }
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
            "if" => TokenType::If,
            "let" => TokenType::Let,
            "proc" => TokenType::Proc,
            "struct" => TokenType::Struct,
            "return" => TokenType::Return,
            "true" | "false" => TokenType::Literal(LiteralType::Bool),
            _ => TokenType::Ident,
        };

        Some(Token::from(token_type, value, pos))
    }

    fn parse_digit_token(&mut self, pos: Position) -> Option<Token> {
        let start = self.cursor;
        let mut c = self.character();

        let mut is_float = false;
        while self.valid() && c.is_ascii_digit() {
            self.advance();
            c = self.character();

            if c == '.' {
                is_float = true;
                self.advance();
                c = self.character();
            }
        }

        let lt = if is_float {
            LiteralType::Float
        } else {
            LiteralType::Number
        };

        let value = String::from(&self.source[start..self.cursor]);
        Some(Token::from(TokenType::Literal(lt), value, pos))
    }
}

fn get_next_token(lexer: &mut Lexer) -> Option<Token> {
    if !lexer.valid() {
        return None;
    }

    if lexer.character().is_ascii_whitespace() {
        lexer.trim();

        if !lexer.valid() {
            return None;
        }
    }

    if lexer.character() == '/' {
        if let Some(c) = lexer.peek_char() {
            if c == '/' {
                lexer.drop_line();
            }
        }
    }

    let first = lexer.character();
    let pos = lexer.get_cursor_pos();

    let punctuation_tokens = "(){};:,.";
    let operator_tokens = "+-*/=<>!";

    if first == '"' {
        lexer.parse_string_token(pos)
    } else if first == '\'' {
        lexer.parse_char_token(pos)
    } else if punctuation_tokens.contains(first) {
        lexer.parse_punctuation_token(pos)
    } else if operator_tokens.contains(first) {
        lexer.parse_operator_token(pos)
    } else if first.is_ascii_alphabetic() || first == '_' {
        lexer.parse_ident_token(pos)
    } else if first.is_ascii_digit() {
        lexer.parse_digit_token(pos)
    } else {
        None
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        get_next_token(self)
    }
}
