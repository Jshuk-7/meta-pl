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
            } else if let Some(expr) = self.parse_expr(token) {
                self.program.push(expr);
            }
        }

        self.write_to_file("ast.dat");
    }

    fn parse_expr(&mut self, token: &Token) -> Option<Expression> {
        type TT = TokenType;

        if self
            .variables
            .iter()
            .find(|&v| v.var.name == token.value)
            .is_some()
            || self
                .functions
                .iter()
                .find(|&f| f.name == token.value)
                .is_some()
        {
            if let Some(ident) = self.visit_identifier(token) {
                return self.visit_binary_op(Some(ident));
            }
        }

        match token.kind.clone() {
            TT::Proc => self.visit_procedure_def(),
            TT::Ident => self.visit_identifier(token),
            TT::Let => self.visit_let_statement(),
            TT::Literal(lt) => {
                let literal = Some(Expression::Literal(token.clone(), lt));
                self.visit_binary_op(literal)
            }
            _ => None,
        }
    }

    fn visit_procedure_def(&mut self) -> Option<Expression> {
        type TT = TokenType;

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
                    if n.kind == TT::Colon {
                        let rt = self.lexer.next().unwrap();
                        return_type = Some(rt.value);

                        let _ocurly = self.lexer.next().unwrap();
                    }

                    while let Some(next) = self.lexer.next() {
                        if let TT::Ccurly = next.kind {
                            break;
                        } else if let TT::Semicolon = next.kind {
                            continue;
                        }

                        if let TT::Return = next.kind {
                            let rv = self.lexer.next().unwrap();
                            if return_type.is_some() {
                                if let Some(value) = self.parse_expr(&rv) {
                                    return_value = Some(Box::new(value));
                                }
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
        while let Some(ident) = self.lexer.next() {
            if let TokenType::Cparen = ident.kind {
                break;
            } else if let TokenType::Comma = ident.kind {
                continue;
            }

            let _colon = self.lexer.next().unwrap();
            let type_name = self.lexer.next().unwrap();

            let kind = self.literal_type_from_string(type_name.value);

            let arg = VarDef {
                name: ident.value,
                kind,
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

                            let mut variable = var.clone();
                            variable.value = new_value.clone();

                            if let Some(pos) = self
                                .variables
                                .iter()
                                .position(|v| v.var.name == variable.var.name)
                            {
                                self.variables.remove(pos);
                                self.variables.insert(pos, variable.clone());
                            }

                            return Some(Expression::AssignStatement {
                                value: variable,
                                new_value,
                            });
                        }
                    }
                } else {
                    return self.visit_binary_op(Some(Expression::Variable(var.clone())));
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

            return self.visit_binary_op(Some(Expression::FunCall {
                proc_def: proc_def.clone(),
                args,
            }));
        } else {
            println!("<{}> Error: expected identifier found '{}'", token.position, token.value);
        }

        None
    }

    fn visit_let_statement(&mut self) -> Option<Expression> {
        if let Some(ident) = self.lexer.next() {
            if let Some(next) = self.lexer.next() {
                let mut type_hint = None;

                if let TokenType::Colon = next.kind {
                    let type_name = self.lexer.next().unwrap();
                    if let TokenType::Ident = type_name.kind {
                        type_hint = Some(type_name.value);
                    }

                    let _equal_op = self.lexer.next().unwrap();
                }

                let first = self.lexer.next().unwrap();

                if let Some(value) = self.parse_expr(&first) {
                    let name = ident.value;
                    let value = Box::new(value);
                    let kind = match first.kind {
                        TokenType::Literal(lt) => lt,
                        TokenType::Ident => {
                            if let Some(var) = self.variables.iter().find(|&v| v.var.name == first.value) {
                                var.var.kind.clone()
                            } else if let Some(proc_def) = self.functions.iter().find(|&f| f.name == first.value) {
                                if let Some(rt) = proc_def.return_type.clone() {
                                    self.literal_type_from_string(rt)
                                } else {
                                    LiteralType::None
                                }
                            } else {
                                LiteralType::None
                            }
                        }
                        _ => LiteralType::None,
                    };

                    if let Some(_type) = type_hint {
                        let kind_str = self.string_from_literal_type(kind.clone());

                        if kind_str != _type {
                            println!(
                                "<{}> Error: expected {_type} found '{kind_str}'",
                                first.position,
                            );
                        }
                    }

                    let variable = self.create_variable(name.clone(), kind, value.clone());
                    self.variables.push(variable);

                    return Some(Expression::LetStatement { name, value });
                }
            }
        }

        None
    }

    fn visit_binary_op(&mut self, expr: Option<Expression>) -> Option<Expression> {
        let mut ex = expr;

        let ops = "+-*/=";
        while let Some(potential_op) = self.lexer.peek_char() {
            if !ops.contains(potential_op) {
                break;
            }

            let op_token = self.lexer.next().unwrap();
            let op = self.token_type_to_binary_op(op_token.kind);

            let next = self.lexer.next().unwrap();
            if let TokenType::Literal(lt) = next.kind.clone() {
                let rhs = Box::new(Expression::Literal(next, lt));
    
                if let Some(lhs) = ex {
                    ex = Some(Expression::BinaryOperation(Box::new(lhs), op, rhs));
                }
            } else if let TokenType::Ident = next.kind.clone() {
                if let Some(var) = self.variables.iter().find(|&v| v.var.name == next.value) {
                    let rhs = Box::new(Expression::Variable(var.clone()));

                    if let Some(lhs) = ex {
                        ex = Some(Expression::BinaryOperation(Box::new(lhs), op, rhs));
                    }
                }
            }
        }

        return ex;
    }

    fn create_variable(&self, name: String, kind: LiteralType, value: Box<Expression>) -> Variable {
        Variable {
            var: VarDef { name, kind },
            value,
        }
    }

    fn string_from_literal_type(&self, kind: LiteralType) -> String {
        let kind = format!("{kind:?}");
        let s = match &kind[..] {
            "Char" => "char",
            "Bool" => "bool",
            "Number" => "i32",
            kind => kind,
        };

        String::from(s)
    }

    fn literal_type_from_string(&self, value: String) -> LiteralType {
        match &value[..] {
            "char" => LiteralType::Char,
            "bool" => LiteralType::Bool,
            "i32" => LiteralType::Number,
            "String" => LiteralType::String,
            _ => LiteralType::None,
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
