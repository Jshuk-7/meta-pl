use std::{fs::File, path::Path};

use crate::{
    expression::{BinaryOp, Expression, ProcDef, VarDef, Variable},
    lexer::Lexer,
    token::{LiteralType, Token, TokenType},
};

pub type Program = Vec<Expression>;

pub struct Parser {
    lexer: Lexer,
    program: Program,
    variables: Vec<Variable>,
    functions: Vec<ProcDef>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            program: Program::new(),
            variables: Vec::new(),
            functions: Vec::new(),
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
            TT::Proc => self.visit_procedure_def(),
            TT::Ident => self.visit_identifier(token),
            TT::Let => self.visit_let_statement(),
            TT::Literal(_) => self.visit_binary_op(token),
            _ => None,
        }
    }

    fn visit_procedure_def(&mut self) -> Option<Expression> {
        if let Some(ident) = self.lexer.next() {
            let mut args = Vec::new();
            let mut statements = Vec::new();

            if let Some(_oparen) = self.lexer.next() {
                // args
                self.visit_args(&mut args);

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

                        if let TokenType::Return = next.kind {
                            let rv = self.lexer.next().unwrap();
                            if let Some(value) = self.parse_expr(&rv) {
                                return_value = Some(Box::new(value));
                            }

                            break;
                        }

                        if let Some(expr) = self.parse_expr(&next) {
                            statements.push(expr);
                        }
                    }
                }

                let proc_def = ProcDef {
                    name: ident.value,
                    return_type,
                    return_value,
                    args,
                    statements,
                };

                self.functions.push(proc_def.clone());

                return Some(Expression::ProcDef(proc_def));
            }
        }

        None
    }

    fn visit_args(&mut self, args: &mut Vec<VarDef>) {
        while let Some(potential_arg) = self.lexer.next() {
            if potential_arg.kind == TokenType::Cparen {
                break;
            }

            if potential_arg.kind == TokenType::Comma {
                continue;
            }

            let _colon = self.lexer.next().unwrap();
            let type_name = self.lexer.next().unwrap();

            let literal_type = match type_name.value.as_str() {
                "char" => LiteralType::Char,
                "i32" => LiteralType::Number,
                "String" => LiteralType::String,
                _ => LiteralType::None,
            };

            let arg = VarDef {
                name: potential_arg.value,
                kind: literal_type,
            };

            args.push(arg);
        }
    }

    fn visit_identifier(&mut self, token: &Token) -> Option<Expression> {
        if let Some(var) = self
            .variables
            .clone()
            .iter()
            .find(|&v| v.var.name == token.value)
        {
            if let Some(c) = self.lexer.peek_char() {
                if c == '=' {
                    if let Some(_equal_op) = self.lexer.next() {
                        let next = self.lexer.next().unwrap();

                        if let Some(expr) = self.parse_expr(&next) {
                            let new_value = Box::new(expr);

                            return Some(Expression::AssignStatement {
                                value: var.clone(),
                                new_value,
                            });
                        }
                    }
                } else {
                    return Some(Expression::Variable(var.clone()));
                }
            }
        } else if let Some(proc_def) = self
            .functions
            .clone()
            .iter()
            .find(|&f| f.name == token.value)
        {
            let mut args = Vec::new();

            if let Some(_oparen) = self.lexer.next() {
                let mut i = 0;
                while let Some(potential_arg) = self.lexer.next() {
                    if potential_arg.kind == TokenType::Cparen {
                        break;
                    } else if potential_arg.kind == TokenType::Comma {
                        continue;
                    }

                    if let Some(value) = self.parse_expr(&potential_arg) {
                        let var = proc_def.args[i].clone();
                        let variable = self.create_variable(var.name, var.kind, Box::new(value));

                        args.push(variable);

                        i += 1;
                    }
                }
            }

            return Some(Expression::FunCall {
                proc_def: proc_def.clone(),
                args,
            });
        } else {
            println!("Error: expected identifier found '{}'", token.value);
        }

        None
    }

    fn visit_let_statement(&mut self) -> Option<Expression> {
        if let Some(ident) = self.lexer.next() {
            // TODO: inline type hints
            println!("{}", self.lexer.peek_char().unwrap());
            if let Some(_equal_op) = self.lexer.next() {
                let first = self.lexer.next().unwrap();

                if let Some(value) = self.parse_expr(&first) {
                    let name = ident.value;
                    let value = Box::new(value);
                    let kind = match first.kind {
                        TokenType::Literal(lt) => lt,
                        _ => LiteralType::None,
                    };

                    let variable = self.create_variable(name.clone(), kind, value.clone());
                    self.variables.push(variable);

                    return Some(Expression::LetStatement { name, value });
                }
            }
        }

        None
    }

    fn visit_binary_op(&mut self, token: &Token) -> Option<Expression> {
        if let TokenType::Literal(literal_type) = token.kind.clone() {
            let start = Some(Expression::Literal(token.clone(), literal_type.clone()));
            let mut ex = start;

            let ops = "+-*/=";
            while let Some(potential_op) = self.lexer.peek_char() {
                if !ops.contains(potential_op) {
                    break;
                }

                let op_token = self.lexer.next().unwrap();
                let op = self.token_type_to_binary_op(op_token.kind);

                let next = self.lexer.next().unwrap();
                let rhs = Box::new(Expression::Literal(next, literal_type.clone()));

                if let Some(lhs) = ex {
                    ex = Some(Expression::BinaryOperation(Box::new(lhs), op, rhs));
                }
            }

            return ex;
        }

        self.visit_literal(token)
    }

    fn visit_literal(&mut self, token: &Token) -> Option<Expression> {
        if let TokenType::Literal(literal_type) = token.kind.clone() {
            return Some(Expression::Literal(token.clone(), literal_type));
        }

        None
    }

    fn create_variable(&self, name: String, kind: LiteralType, value: Box<Expression>) -> Variable {
        Variable {
            var: VarDef { name, kind },
            value,
        }
    }

    fn token_type_to_binary_op(&self, kind: TokenType) -> BinaryOp {
        match kind {
            TokenType::Plus => BinaryOp::Plus,
            TokenType::Minus => BinaryOp::Minus,
            TokenType::Multiply => BinaryOp::Multiply,
            TokenType::Divide => BinaryOp::Divide,
            _ => BinaryOp::Plus,
        }
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
