use std::{any::Any, borrow::BorrowMut, fs::File, path::Path, string::ParseError};

use crate::{
    expression::Expression,
    lexer::Lexer,
    nodes::{
        AssignNode, BinaryOp, BinaryOpNode, FunCallNode, IfNode, LetNode, ProcDefNode, ReturnNode,
        StructDefNode, StructInstanceNode, VarMetadataNode, VariableNode,
    },
    token::{LiteralType, Token, TokenType},
};

pub type Program = Vec<Expression>;

pub struct Parser {
    lexer: Lexer,
    program: Program,
    variables: Vec<VariableNode>,
    procedures: Vec<ProcDefNode>,
    structs: Vec<StructDefNode>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            program: Program::new(),
            variables: Vec::new(),
            procedures: Vec::new(),
            structs: Vec::new(),
        }
    }

    pub fn from_file<P: AsRef<Path> + Clone>(path: P) -> std::io::Result<Self> {
        let source = std::fs::read_to_string(path.clone())?;
        let filename = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        let lexer = Lexer::new(source, filename);
        let this = Self::new(lexer);

        Ok(this)
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        while let Some(token) = &self.lexer.next() {
            if let Some(expr) = self.parse_expr(token) {
                self.program.push(expr);
            }
        }

        self.write_to_file("ast.dat");
        Ok(self.program.clone())
    }

    fn parse_expr(&mut self, token: &Token) -> Option<Expression> {
        type TT = TokenType;

        match token.kind {
            TT::If => self.visit_if_statement(),
            TT::Let => self.visit_let_statement(),
            TT::Return => self.visit_return_statement(),
            TT::Proc => self.visit_procedure_def(),
            TT::Ident => self.visit_identifier(token),
            TT::Struct => self.visit_struct_def(),
            TT::Literal(lt) => {
                let literal = Some(Expression::Literal(token.clone(), lt));
                self.visit_binary_op(literal)
            }
            _ => None,
        }
    }

    fn visit_if_statement(&mut self) -> Option<Expression> {
        let first = self.lexer.next().unwrap();
        if let Some(expr) = self.parse_expr(&first) {
            let binary_op = match expr {
                Expression::FunCall(_) => {
                    // TODO verify that the proc has a return type and it is of type bool
                    None
                }
                Expression::Variable(mut var) => {
                    if let LiteralType::Bool = var.metadata.kind {
                        let v: &mut Expression = var.value.borrow_mut();
                        let any: &dyn Any = v;
                        match any.downcast_ref::<Expression>() {
                            Some(expr) => self.visit_binary_op(Some(expr.clone())),
                            None => None,
                        }
                    } else {
                        None
                    }
                }
                Expression::BinaryOp(_) => Some(expr),
                Expression::Literal(..) => self.visit_binary_op(Some(expr)),
                _ => None,
            };

            binary_op.as_ref()?;

            if let Some(_ocurly) = self.lexer.next() {
                let mut statements = Vec::new();

                let mut next = self.lexer.next().unwrap();
                while self.lexer.valid() {
                    if let TokenType::Ccurly = next.kind {
                        break;
                    } else if let TokenType::Semicolon = next.kind {
                        next = self.lexer.next().unwrap();
                        continue;
                    }

                    if let Some(expr) = self.parse_expr(&next) {
                        statements.push(expr.clone());

                        if let Some(n) = self.lexer.next() {
                            next = n;
                        }
                    }
                }

                let if_node = IfNode {
                    value: Box::new(binary_op.unwrap()),
                    statements,
                };

                return Some(Expression::IfStatement(if_node));
            }
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
                    let mut kind_str = String::new();

                    let kind = match first.kind {
                        TokenType::Literal(lt) => {
                            kind_str = self.string_from_literal_type(lt);
                            lt
                        },
                        TokenType::Ident => {
                            if let Some(var) = self
                                .variables
                                .iter()
                                .find(|&v| v.metadata.name == first.value)
                            {
                                kind_str = self.string_from_literal_type(var.metadata.kind);
                                var.metadata.kind
                            } else if let Some(proc_def) =
                                self.procedures.iter().find(|&f| f.name == first.value)
                            {
                                if let Some(rt) = proc_def.return_type.clone() {
                                    let lt = self.literal_type_from_string(rt.clone());
                                    kind_str = rt;
                                    lt
                                } else {
                                    LiteralType::None
                                }
                            } else if let Some(struct_def) =
                                self.structs.iter().find(|&s| s.type_name == first.value)
                            {
                                kind_str = struct_def.type_name.clone();
                                LiteralType::None
                            } else {
                                LiteralType::None
                            }
                        }
                        _ => LiteralType::None,
                    };

                    if let Some(_type) = type_hint {
                        if kind_str != _type {
                            println!(
                                "<{}> Error: expected {_type} found '{kind_str}'",
                                first.position,
                            );
                        }
                    }

                    let variable = self.make_variable(name.clone(), kind, value.clone());
                    self.variables.push(variable);

                    let let_node = LetNode { name, kind, value };

                    return Some(Expression::LetStatement(let_node));
                }
            }
        }

        None
    }

    fn visit_return_statement(&mut self) -> Option<Expression> {
        if let Some(first) = self.lexer.next() {
            if let Some(return_value) = self.parse_expr(&first) {
                let return_node = ReturnNode {
                    value: Box::new(return_value),
                };

                return Some(Expression::ReturnStatement(return_node));
            }
        }

        None
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

                        if let Some(expr) = self.parse_expr(&next) {
                            statements.push(expr);
                        } else {
                            break;
                        }
                    }
                }

                for arg in args.clone().iter() {
                    let pos = self
                        .variables
                        .iter()
                        .position(|v| v.metadata.name == arg.name)
                        .unwrap();

                    self.variables.remove(pos);
                }

                let proc_def_node = ProcDefNode {
                    name: ident.value,
                    return_type,
                    args,
                    statements,
                };

                self.procedures.push(proc_def_node.clone());

                return Some(Expression::ProcDef(proc_def_node));
            }
        }

        None
    }

    fn visit_args(&mut self, args: &mut Vec<VarMetadataNode>) {
        while let Some(ident) = self.lexer.next() {
            if let TokenType::Cparen = ident.kind {
                break;
            } else if let TokenType::Comma = ident.kind {
                continue;
            }

            let _colon = self.lexer.next().unwrap();
            let type_name = self.lexer.next().unwrap();

            let kind = self.literal_type_from_string(type_name.value);

            let arg = VarMetadataNode {
                name: ident.value,
                kind,
            };

            args.push(arg.clone());

            let value = self.default_construct_value(kind);
            let var = VariableNode {
                metadata: arg,
                value,
            };
            self.variables.push(var);
        }
    }

    fn visit_identifier(&mut self, token: &Token) -> Option<Expression> {
        if let Some(var) = self
            .variables
            .clone()
            .iter()
            .find(|&v| v.metadata.name == token.value)
        {
            if let Some(c) = self.lexer.peek_char() {
                if c == '=' {
                    if let Some(_equal_op) = self.lexer.next() {
                        let next = self.lexer.next().unwrap();

                        if let Some(expr) = self.parse_expr(&next) {
                            let new_value = Box::new(expr);

                            let assign_node = AssignNode {
                                value: var.clone(),
                                new_value,
                            };

                            return Some(Expression::AssignStatement(assign_node));
                        }
                    }
                } else {
                    return self.visit_binary_op(Some(Expression::Variable(var.clone())));
                }
            }
        } else if let Some(proc_def) = self
            .procedures
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
                        let variable = self.make_variable(var.name, var.kind, Box::new(value));

                        args.push(variable);

                        i += 1;
                    }
                }
            }

            let fun_call_node = FunCallNode {
                proc_def: proc_def.clone(),
                args,
            };

            return self.visit_binary_op(Some(Expression::FunCall(fun_call_node)));
        } else if let Some(struct_def) = self
            .structs
            .clone()
            .iter()
            .find(|&s| s.type_name == token.value)
        {
            if let Some(_ocurly) = self.lexer.next() {
                let mut fields = Vec::new();
                let mut i = 0;

                while self.lexer.valid() {
                    if let Some(field) = self.lexer.next() {
                        if let TokenType::Ccurly = field.kind {
                            break;
                        } else if field.kind != TokenType::Ident {
                            println!(
                                "<{}> Error: expected identifier found '{:?}'",
                                field.position, field.kind
                            );

                            break;
                        }

                        let _colon = self.lexer.next().unwrap();

                        let first = self.lexer.next().unwrap();
                        if let Some(value) = self.parse_expr(&first) {
                            let field = self.make_variable(
                                struct_def.fields[i].name.clone(),
                                struct_def.fields[i].kind,
                                Box::new(value),
                            );

                            fields.push(field);
                            i += 1;
                        }
                    }

                    if self.lexer.character() == ',' {
                        let _comma = self.lexer.next().unwrap();
                    }

                    if let Some(c) = self.lexer.peek_char() {
                        if c == '}' {
                            let _ccurly = self.lexer.next().unwrap();
                            break;
                        }
                    }
                }

                let _semicolon = self.lexer.next().unwrap();

                let struct_instance_node = StructInstanceNode {
                    struct_def: struct_def.clone(),
                    fields,
                };

                return Some(Expression::StructInstance(struct_instance_node));
            }
        } else {
            println!(
                "<{}> Error: expected identifier found '{}'",
                token.position, token.value
            );
        }

        None
    }

    fn visit_struct_def(&mut self) -> Option<Expression> {
        if let Some(ident) = self.lexer.next() {
            if let Some(_ocurly) = self.lexer.next() {
                let mut fields = Vec::new();

                while self.lexer.valid() {
                    if let Some(field) = self.lexer.next() {
                        if let TokenType::Ccurly = field.kind {
                            break;
                        } else if field.kind != TokenType::Ident {
                            println!(
                                "<{}> Error: expected identifier found '{:?}'",
                                field.position, field.kind
                            );

                            break;
                        }

                        let _colon = self.lexer.next().unwrap();

                        if let Some(type_name) = self.lexer.next() {
                            let literal_type = self.literal_type_from_string(type_name.value);
                            let var = VarMetadataNode {
                                name: field.value,
                                kind: literal_type,
                            };

                            fields.push(var);
                        }

                        if self.lexer.character() == ',' {
                            let _comma = self.lexer.next().unwrap();
                        }
                    }
                }

                if let Some(c) = self.lexer.peek_char() {
                    if c == '}' {
                        let _ccurly = self.lexer.next().unwrap();
                    }
                }

                let struct_def = StructDefNode {
                    type_name: ident.value,
                    fields,
                };

                self.structs.push(struct_def.clone());

                return Some(Expression::StructDef(struct_def));
            }
        }

        None
    }

    fn visit_binary_op(&mut self, expr: Option<Expression>) -> Option<Expression> {
        let mut ex = expr;

        let ops = "+-*/=<>!";
        while let Some(potential_op) = self.lexer.peek_char() {
            if !ops.contains(potential_op) {
                break;
            }

            let op_token = self.lexer.next().unwrap();
            let op = self.token_type_to_binary_op(op_token.kind);

            let next = self.lexer.next().unwrap();
            if let TokenType::Literal(lt) = next.kind {
                let rhs = Box::new(Expression::Literal(next, lt));

                if let Some(lhs) = ex {
                    let binary_op_node = BinaryOpNode {
                        lhs: Box::new(lhs),
                        op,
                        rhs,
                    };

                    ex = Some(Expression::BinaryOp(binary_op_node));
                }
            } else if let TokenType::Ident = next.kind {
                if let Some(var) = self
                    .variables
                    .iter()
                    .find(|&v| v.metadata.name == next.value)
                {
                    let rhs = Box::new(Expression::Variable(var.clone()));

                    if let Some(lhs) = ex {
                        let binary_op = BinaryOpNode {
                            lhs: Box::new(lhs),
                            op,
                            rhs,
                        };

                        ex = Some(Expression::BinaryOp(binary_op));
                    }
                }
            }
        }

        ex
    }

    fn default_construct_value(&self, kind: LiteralType) -> Box<Expression> {
        let token = match kind {
            LiteralType::Char => Token::from(
                TokenType::Literal(kind),
                String::from(""),
                self.lexer.get_cursor_pos(),
            ),
            LiteralType::Bool => Token::from(
                TokenType::Literal(kind),
                String::from("false"),
                self.lexer.get_cursor_pos(),
            ),
            LiteralType::Number => Token::from(
                TokenType::Literal(kind),
                String::from("0"),
                self.lexer.get_cursor_pos(),
            ),
            LiteralType::Float => Token::from(
                TokenType::Literal(kind),
                String::from("0.0"),
                self.lexer.get_cursor_pos(),
            ),
            LiteralType::String => Token::from(
                TokenType::Literal(kind),
                String::from(""),
                self.lexer.get_cursor_pos(),
            ),
            _ => panic!("unimplemented literal type"),
        };

        let expr = Expression::Literal(token, kind);
        Box::new(expr)
    }

    fn make_variable(
        &self,
        name: String,
        kind: LiteralType,
        value: Box<Expression>,
    ) -> VariableNode {
        VariableNode {
            metadata: VarMetadataNode { name, kind },
            value,
        }
    }

    fn string_from_literal_type(&self, kind: LiteralType) -> String {
        let kind = format!("{kind:?}");
        let s = match &kind[..] {
            "Char" => "char",
            "Bool" => "bool",
            "Number" => "i32",
            "Float" => "f32",
            kind => kind,
        };

        String::from(s)
    }

    fn literal_type_from_string(&self, value: String) -> LiteralType {
        match &value[..] {
            "char" => LiteralType::Char,
            "bool" => LiteralType::Bool,
            "i32" => LiteralType::Number,
            "f32" => LiteralType::Float,
            "String" => LiteralType::String,
            _ => LiteralType::None,
        }
    }

    fn token_type_to_binary_op(&self, kind: TokenType) -> BinaryOp {
        type TT = TokenType;
        match kind {
            TT::Add => BinaryOp::Add,
            TT::Sub => BinaryOp::Sub,
            TT::Mul => BinaryOp::Mul,
            TT::Div => BinaryOp::Div,
            TT::Eq => BinaryOp::Eq,
            TT::Ne => BinaryOp::Ne,
            TT::Lt => BinaryOp::Lt,
            TT::Lte => BinaryOp::Lte,
            TT::Gt => BinaryOp::Gt,
            TT::Gte => BinaryOp::Gte,
            TT::Neg => BinaryOp::Neg,
            _ => BinaryOp::None,
        }
    }

    fn write_to_file<P: AsRef<Path>>(&self, path: P) {
        let mut content = String::new();

        if let Ok(mut file) = File::create(path) {
            use std::fmt::Write;
            use std::io::Write as W;

            /* for custom ast */
            for expr in self.program.iter() {
                content.write_fmt(format_args!("{}\n", expr)).unwrap();
            }

            // content
            //     .write_fmt(format_args!("{program:#?}", program = self.program))
            //     .unwrap();

            file.write_all(content.as_bytes()).unwrap();
        }
    }
}
