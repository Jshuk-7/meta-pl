use std::fmt::Display;

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

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum LiteralType {
    #[default]
    None,
    Number,
    String,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum TokenType {
    #[default]
    None,
    Proc,
    Ident,
    Let,
    Return,
    Oparen,
    Cparen,
    Colon,
    Semicolon,
    Comma,
    Ocurly,
    Ccurly,
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    Literal(LiteralType),
}

#[derive(Debug, Default, Clone)]
pub struct Token {
    pub _type: TokenType,
    pub value: String,
    pub position: Position,
}

impl Token {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(_type: TokenType, value: String, position: Position) -> Self {
        Self {
            _type,
            value,
            position,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{} {:?}> {}",
            self.position, self._type, self.value
        ))
    }
}
