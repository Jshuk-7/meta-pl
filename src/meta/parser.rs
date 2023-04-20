use std::{fs::File, path::Path};

use crate::{
    expression::{Var, BinaryOp, Expression},
    lexer::Lexer,
    token::{Token, TokenType},
};

pub type Program = Vec<Expression>;

pub struct Parser {
    lexer: Lexer,
    program: Program,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            program: Program::new(),
        }
    }

    pub fn from_file<P: AsRef<Path> + Clone>(path: P) -> Option<Self> {
        if let Ok(source) = std::fs::read_to_string(path.clone()) {
            let lexer = Lexer::new(
                source,
                path.as_ref()
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap(),
            );

            return Some(Self::new(lexer));
        }

        None
    }

    pub fn make_program(&mut self) {
        while let Some(token) = &self.lexer.next() {
            if let TokenType::Semicolon = token.kind {
                continue;
            }

            if let Some(expr) = self.parse_expr(token) {
                self.program.push(expr);
            }
        }

        self.write_to_file("ast.dat");
    }

    fn parse_expr(&mut self, token: &Token) -> Option<Expression> {
        type TT = TokenType;

        match token.kind {
            TT::Proc => self.parse_procedure(),
            TT::Ident => self.parse_identifier(token),
            TT::Let => self.parse_let_expr(),
            TT::Literal(_) => self.parse_binary_op(token),
            _ => None,
        }
    }

    fn parse_procedure(&mut self) -> Option<Expression> {
        if let Some(ident) = self.lexer.next() {
            let mut args = Vec::new();
            let mut statements = Vec::new();

            if let Some(_oparen) = self.lexer.next() {
                // args
                self.parse_args(&mut args);

                let mut return_type = None;
                let mut return_value = None;

                // statements
                if let Some(n) = self.lexer.next() {
                    if n.kind == TokenType::Colon {
                        let rt = self.lexer.next().unwrap();
                        return_type = Some(rt.value);

                        let _ocurly = self.lexer.next().unwrap();
                    }

                    type TT = TokenType;
                    while let Some(next) = self.lexer.next() {
                        if let TT::Ccurly = next.kind {
                            break;
                        } else if let TT::Semicolon = next.kind {
                            continue;
                        }

                        if next.kind == TokenType::Return {
                            let rt = self.lexer.next().unwrap();
                            if let Some(value) = self.parse_expr(&rt) {
                                return_value = Some(Box::new(value));
                            }

                            break;
                        }

                        if let Some(expr) = self.parse_expr(&next) {
                            statements.push(expr);
                        }
                    }
                }

                if let Some(next) = self.lexer.next() {
                    if next.kind == TokenType::Semicolon {
                        let _ccurly = self.lexer.next().unwrap();
                    }
                }

                return Some(Expression::ProcDef {
                    name: ident.value,
                    args,
                    statements,
                    return_type,
                    return_value,
                });
            }
        }

        None
    }

    fn parse_args(&mut self, args: &mut Vec<Var>) {
        while let Some(potential_arg) = self.lexer.next() {
            if potential_arg.kind == TokenType::Cparen {
                break;
            }

            if potential_arg.kind == TokenType::Comma {
                continue;
            }

            let _colon = self.lexer.next().unwrap();
            let type_name = self.lexer.next().unwrap();

            let arg = Var {
                name: potential_arg.value,
                kind: type_name.value,
            };

            args.push(arg);
        }
    }

    fn parse_identifier(&mut self, token: &Token) -> Option<Expression> {
        println!("{token}");
        None
    }

    fn parse_let_expr(&mut self) -> Option<Expression> {
        if let Some(ident) = self.lexer.next() {
            if let Some(_equal_op) = self.lexer.next() {
                let first = self.lexer.next().unwrap();

                if let Some(value) = self.parse_expr(&first) {
                    return Some(Expression::LetStatement {
                        name: ident.value,
                        value: Box::new(value),
                    });
                }
            }
        }

        None
    }

    fn parse_binary_op(&mut self, token: &Token) -> Option<Expression> {
        let mut ex = None;

        if let TokenType::Literal(literal_type) = token.clone().kind {
            let start = Some(Expression::Literal(token.clone(), literal_type.clone()));
            ex = start;

            let ops = "+-*/=";
            while let Some(potential_op) = self.lexer.peek_char() {
                if !ops.contains(potential_op) {
                    break;
                }

                let op_token = self.lexer.next().unwrap();
                let op = match op_token.kind {
                    TokenType::Plus => BinaryOp::Plus,
                    TokenType::Minus => BinaryOp::Minus,
                    TokenType::Multiply => BinaryOp::Multiply,
                    TokenType::Divide => BinaryOp::Divide,
                    _ => BinaryOp::Plus,
                };

                let next = self.lexer.next().unwrap();
                let rhs = Box::new(Expression::Literal(next, literal_type.clone()));

                if let Some(lhs) = ex {
                    ex = Some(Expression::BinaryOperation(Box::new(lhs), op, rhs));
                }
            }
        }

        ex
    }

    fn parse_literal(&mut self, token: &Token) -> Option<Expression> {
        if let TokenType::Literal(literal_type) = token.clone().kind {
            return Some(Expression::Literal(token.clone(), literal_type));
        }

        None
    }

    fn write_to_file<P: AsRef<Path>>(&self, path: P) {
        let mut content = String::new();

        if let Ok(mut file) = File::create(path) {
            use std::fmt::Write;
            use std::io::Write as W;

            for expr in self.program.iter() {
                content.write_fmt(format_args!("{}\n", expr)).unwrap();
            }

            file.write_all(content.as_bytes()).unwrap();
        }
    }
}
