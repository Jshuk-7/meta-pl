use std::fmt::Display;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum LiteralType {
    #[default]
    None,
    Char,
    Bool,
    Number,
    Float,
    String,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    #[default]
    None,
    If,
    While,
    For,
    In,
    Range,
    Let,
    Proc,
    Ident,
    Struct,
    Return,
    Oparen,
    Cparen,
    Colon,
    Semicolon,
    Comma,
    Period,
    Ocurly,
    Ccurly,
    Add,
    Sub,
    Mul,
    Div,
    Assign,
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Neg,
    Literal(LiteralType),
}

#[derive(Debug, Default, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub value: String,
    pub position: Position,
}

impl Token {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(_type: TokenType, value: String, position: Position) -> Self {
        Self {
            kind: _type,
            value,
            position,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{} {:?}> {}",
            self.position, self.kind, self.value
        ))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Position {
    pub filename: String,
    pub row: u32,
    pub column: u32,
}

impl Position {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(filename: String, row: u32, column: u32) -> Self {
        Self {
            filename,
            row,
            column,
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}:{}",
            self.filename,
            self.row + 1,
            self.column + 1
        ))
    }
}
