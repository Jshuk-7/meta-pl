use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Position {
    filename: String,
    row: u32,
    column: u32,
}

impl Position {
    pub fn new() -> Self {
        Self {
            filename: String::new(),
            row: 0,
            column: 0,
        }
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

#[derive(Debug, Clone)]
pub enum TokenType {
    None,
    Proc,
    Ident,
    Oparen,
    Cparen,
    Colon,
    Semicolon,
    Ocurly,
    Ccurly,
}

#[derive(Debug, Clone)]
pub struct Token {
    ty: TokenType,
    value: String,
    position: Position,
}

impl Token {
    pub fn new() -> Self {
        Self {
            ty: TokenType::None,
            value: String::new(),
            position: Position::new(),
        }
    }

    pub fn from(ty: TokenType, value: String, position: Position) -> Self {
        Self {
            ty,
            value,
            position,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<{} {:?}> {}",
            self.position, self.ty, self.value
        ))
    }
}
