use std::{fs::File, path::Path, string::ParseError};

use crate::{
    expression::Expression,
    lexer::Lexer,
    nodes::{
        AssignNode, BinaryOp, BinaryOpNode, FieldAccessNode, FieldAssignNode, ForNode, FunCallNode,
        IfNode, ImplFunCallNode, ImplNode, LetNode, ProcDefNode, RangeNode, ReturnNode,
        StructDefNode, StructInstanceNode, VarMetadataNode, VariableNode, WhileNode,
    },
    timer::Timer,
    token::{LiteralType, Token, TokenType},
};

pub type Program = Vec<Expression>;

pub struct Parser {
    lexer: Lexer,
    program: Program,
    variables: Vec<VariableNode>,
    procedures: Vec<ProcDefNode>,
    structs: Vec<StructDefNode>,
    struct_instances: Vec<StructInstanceNode>,
    impl_blocks: Vec<ImplNode>,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Self {
            lexer,
            program: Program::new(),
            variables: Vec::new(),
            procedures: Vec::new(),
            structs: Vec::new(),
            struct_instances: Vec::new(),
            impl_blocks: Vec::new(),
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
        {
            let _timer = Timer::start("Parsing");

            while let Some(token) = &self.lexer.next() {
                if let Some(expr) = self.parse_expr(token) {
                    self.program.push(expr);
                }
            }
        }

        self.write_to_file("ast.dat");
        Ok(self.program.clone())
    }

    fn parse_expr(&mut self, token: &Token) -> Option<Expression> {
        type TT = TokenType;

        match token.kind {
            TT::If => self.visit_if_statement(),
            TT::While => self.visit_while_statement(),
            TT::For => self.visit_for_loop(),
            TT::Let => self.visit_let_statement(),
            TT::Impl => self.visit_impl_block(),
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
            let boolean_expr = self.visit_boolean_expr(expr);

            boolean_expr.as_ref()?;

            if let Some(_ocurly) = self.lexer.next() {
                let mut statements = Vec::new();

                while let Some(next) = self.lexer.next() {
                    if let TokenType::Ccurly = next.kind {
                        break;
                    } else if let TokenType::Semicolon = next.kind {
                        continue;
                    }

                    if let Some(expr) = self.parse_expr(&next) {
                        statements.push(expr.clone());
                    }
                }

                let if_node = IfNode {
                    value: Box::new(boolean_expr.unwrap()),
                    statements,
                };

                return Some(Expression::IfStatement(if_node));
            }
        }

        None
    }

    fn visit_while_statement(&mut self) -> Option<Expression> {
        let first = self.lexer.next().unwrap();
        if let Some(expr) = self.parse_expr(&first) {
            let boolean_expr = self.visit_boolean_expr(expr);

            boolean_expr.as_ref()?;

            if let Some(_ocurly) = self.lexer.next() {
                let mut statements = Vec::new();

                while let Some(next) = self.lexer.next() {
                    if let TokenType::Ccurly = next.kind {
                        break;
                    } else if let TokenType::Semicolon = next.kind {
                        continue;
                    }

                    if let Some(expr) = self.parse_expr(&next) {
                        statements.push(expr.clone());
                    }
                }

                let while_node = WhileNode {
                    value: Box::new(boolean_expr.unwrap()),
                    statements,
                };

                return Some(Expression::WhileStatement(while_node));
            }
        }

        None
    }

    fn visit_for_loop(&mut self) -> Option<Expression> {
        if let Some(counter_token) = self.lexer.next() {
            let _in = self.lexer.next().unwrap();

            let start_token = self.lexer.next().unwrap();

            let start;
            let end;

            if let Some(s) = self.parse_expr(&start_token) {
                let _range_op = self.lexer.next().unwrap();
                start = Box::new(s);

                let initial_counter_value = start.clone();
                let counter = self.make_variable(
                    counter_token.value,
                    "i32".to_string(),
                    initial_counter_value,
                );

                self.variables.push(counter.clone());
                let counter_index = self.variables.len() - 1;

                let end_token = self.lexer.next().unwrap();
                if let Some(e) = self.parse_expr(&end_token) {
                    end = Box::new(e);

                    let range_node = RangeNode { start, end };
                    let range = Box::new(Expression::RangeStatement(range_node));

                    if let Some(_ocurly) = self.lexer.next() {
                        let mut statements = Vec::new();

                        while let Some(next) = self.lexer.next() {
                            if let TokenType::Ccurly = next.kind {
                                break;
                            } else if let TokenType::Semicolon = next.kind {
                                continue;
                            }

                            if let Some(statement) = self.parse_expr(&next) {
                                statements.push(statement);
                            }
                        }

                        let for_node = ForNode {
                            counter,
                            range,
                            statements,
                        };

                        self.variables.remove(counter_index);

                        return Some(Expression::ForLoop(for_node));
                    }
                }
            }
        }

        None
    }

    fn visit_boolean_expr(&mut self, expr: Expression) -> Option<Expression> {
        match expr.clone() {
            Expression::FunCall(fun_call_node) => {
                if let Some(return_type) = fun_call_node.proc_def.return_type {
                    if return_type == "bool" {
                        return self.visit_binary_op(Some(expr));
                    }
                }

                None
            }
            Expression::Variable(variable_node) => {
                if variable_node.metadata.type_name == "bool" {
                    return self.visit_binary_op(Some(expr));
                }

                None
            }
            Expression::StructFieldAccess(..) => self.visit_binary_op(Some(expr)),
            Expression::BinaryOp(..) => Some(expr),
            Expression::Literal(..) => self.visit_binary_op(Some(expr)),
            _ => None,
        }
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

                    let kind_str = match first.kind {
                        TokenType::Literal(lt) => self.string_from_literal_type(lt),
                        TokenType::Ident => {
                            if let Some(var) = self
                                .variables
                                .iter()
                                .find(|&v| v.metadata.name == first.value)
                            {
                                var.metadata.type_name.clone()
                            } else if let Some(proc_def) =
                                self.procedures.iter().find(|&f| f.name == first.value)
                            {
                                if let Some(return_type) = proc_def.return_type.clone() {
                                    return_type
                                } else {
                                    "None".to_string()
                                }
                            } else if let Some(struct_def) =
                                self.structs.iter().find(|&s| s.type_name == first.value)
                            {
                                struct_def.type_name.clone()
                            } else {
                                "None".to_string()
                            }
                        }
                        _ => "None".to_string(),
                    };

                    if let Some(hint) = type_hint {
                        if kind_str != hint {
                            println!(
                                "<{}> Error: expected '{hint}' found '{kind_str}'",
                                first.position,
                            );
                        }
                    }

                    let variable =
                        self.make_variable(name.clone(), kind_str.clone(), value.clone());
                    self.variables.push(variable);

                    let let_node = LetNode {
                        name,
                        type_name: kind_str,
                        value,
                    };

                    return Some(Expression::LetStatement(let_node));
                }
            }
        }

        None
    }

    fn visit_impl_block(&mut self) -> Option<Expression> {
        if let Some(type_name) = self.lexer.next() {
            if let Some(struct_def) = self
                .structs
                .clone()
                .iter()
                .find(|&s| s.type_name == type_name.value)
            {
                let mut procedures = Vec::new();

                while let Some(next) = self.lexer.next() {
                    if let TokenType::Ccurly = next.kind {
                        break;
                    } else if let TokenType::Semicolon = next.kind {
                        continue;
                    }

                    if let TokenType::Proc = next.kind {
                        if let Some(proc_def_node) = self.parse_expr(&next) {
                            procedures.push(proc_def_node);
                        }
                    }
                }

                let impl_node = ImplNode {
                    procedures,
                    struct_def: struct_def.clone(),
                };

                self.impl_blocks.push(impl_node.clone());

                return Some(Expression::ImplStatement(impl_node));
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

            let arg = VarMetadataNode {
                name: ident.value,
                type_name: type_name.value.clone(),
            };

            args.push(arg.clone());

            let value = self.default_initialize_value(type_name.value);
            let var = VariableNode {
                metadata: arg,
                value: Box::new(value),
            };

            self.variables.push(var);
        }
    }

    fn visit_identifier(&mut self, token: &Token) -> Option<Expression> {
        if let Some(variable) = self
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
                                value: variable.clone(),
                                new_value,
                            };

                            let _semicolon = self.lexer.next().unwrap();

                            return Some(Expression::AssignStatement(assign_node));
                        }
                    }
                }
            }

            if self.lexer.character() == '.' {
                let _period = self.lexer.next().unwrap();
                let expr = self.visit_struct_field(variable);
                return self.visit_binary_op(expr);
            } else {
                return self.visit_binary_op(Some(Expression::Variable(variable.clone())));
            }
        } else if let Some(proc_def) = self
            .procedures
            .clone()
            .iter()
            .find(|&f| f.name == token.value)
        {
            let expr = self.visit_procedure(proc_def);
            return self.visit_binary_op(expr);
        } else if let Some(struct_def) = self
            .structs
            .clone()
            .iter()
            .find(|&s| s.type_name == token.value)
        {
            if self.lexer.character() == ':' {
                if let Some(n) = self.lexer.peek_char() {
                    if n == ':' {
                        if let Some(impl_node) = self
                            .impl_blocks
                            .clone()
                            .iter()
                            .find(|&i| i.struct_def.type_name == token.value)
                        {
                            let expr = self.visit_struct_impl(impl_node);
                            return self.visit_binary_op(expr);
                        }
                    }
                }
            } else {
                let expr = self.make_struct_instance(struct_def);
                return self.visit_binary_op(expr);
            }
        }

        println!(
            "<{}> Error: expected identifier found '{}'",
            token.position, token.value
        );

        None
    }

    fn visit_struct_field(&mut self, variable: &VariableNode) -> Option<Expression> {
        if let Some(struct_field) = self.lexer.next() {
            if let Expression::StructInstance(struct_instance) = variable.value.as_ref() {
                for field in struct_instance.fields.iter() {
                    if field.metadata.name != struct_field.value {
                        continue;
                    }

                    if let Some(c) = self.lexer.peek_char() {
                        let mut is_eq_node = false;

                        if let Some(n) = self.lexer.peek_char_by_amount(2) {
                            is_eq_node = n == '=';
                        }

                        if c == '=' && !is_eq_node {
                            let _equal_op = self.lexer.next().unwrap();

                            let next = self.lexer.next().unwrap();
                            if let Some(value) = self.parse_expr(&next) {
                                let new_value = Box::new(value);

                                let field_assign_node = FieldAssignNode {
                                    struct_instance: variable.clone(),
                                    field: field.clone(),
                                    new_value: new_value.clone(),
                                };

                                if let Expression::StructInstance(struct_instance_node) =
                                    variable.value.as_ref()
                                {
                                    for (i, field) in
                                        struct_instance_node.fields.clone().iter().enumerate()
                                    {
                                        if field.metadata.name
                                            != field_assign_node.field.metadata.name
                                        {
                                            continue;
                                        }

                                        let index = self
                                            .variables
                                            .iter()
                                            .position(|v| v.metadata.name == variable.metadata.name)
                                            .unwrap();
                                        let var = self.variables[index].value.as_mut();
                                        if let Expression::StructInstance(instance) = var {
                                            instance.fields[i].value = new_value.clone();
                                        }
                                    }
                                }

                                return Some(Expression::StructFieldAssign(field_assign_node));
                            }
                        } else {
                            let field_access_node = FieldAccessNode {
                                struct_instance: variable.clone(),
                                field: field.clone(),
                            };

                            return Some(Expression::StructFieldAccess(field_access_node));
                        }
                    }
                }
            }
        }

        None
    }

    fn visit_procedure(&mut self, proc_def: &ProcDefNode) -> Option<Expression> {
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
                    let variable = self.make_variable(var.name, var.type_name, Box::new(value));

                    args.push(variable);

                    i += 1;
                }
            }
        }

        let fun_call_node = FunCallNode {
            proc_def: proc_def.clone(),
            args,
        };

        Some(Expression::FunCall(fun_call_node))
    }

    fn visit_struct_impl(&mut self, impl_node: &ImplNode) -> Option<Expression> {
        if let Some(_scope_resolution) = self.lexer.next() {
            if let Some(proc_name) = self.lexer.next() {
                let mut proc_def = None;

                for proc in impl_node.procedures.iter() {
                    if let Expression::ProcDef(proc_def_node) = proc {
                        if proc_def_node.name == proc_name.value {
                            proc_def = Some(proc_def_node.clone());
                            break;
                        }
                    }
                }

                proc_def.as_ref()?;

                let mut args = Vec::new();
                let mut arg_index = 0;

                if let Some(_oparen) = self.lexer.next() {
                    while let Some(potential_arg) = self.lexer.next() {
                        if let TokenType::Cparen = potential_arg.kind {
                            break;
                        } else if let TokenType::Semicolon | TokenType::Comma = potential_arg.kind {
                            continue;
                        }

                        if let Some(proc) = proc_def.clone() {
                            let name = proc.args[arg_index].name.clone();
                            let type_name = proc.args[arg_index].type_name.clone();

                            if let Some(value) = self.parse_expr(&potential_arg) {
                                let variable = self.make_variable(name, type_name, Box::new(value));

                                args.push(variable);

                                arg_index += 1;
                            }
                        }
                    }

                    let fun_call_node = FunCallNode {
                        proc_def: proc_def.unwrap(),
                        args,
                    };

                    let impl_fun_call_node = ImplFunCallNode {
                        impl_node: impl_node.clone(),
                        fun_call_node: Box::new(Expression::FunCall(fun_call_node)),
                    };

                    let _semicolon = self.lexer.next().unwrap();

                    return Some(Expression::ImplFunCall(impl_fun_call_node));
                }
            }
        }

        None
    }

    fn make_struct_instance(&mut self, struct_def: &StructDefNode) -> Option<Expression> {
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
                        let name = struct_def.fields[i].name.clone();
                        let type_name = struct_def.fields[i].type_name.clone();

                        let field = self.make_variable(name, type_name, Box::new(value));

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

            self.struct_instances.push(struct_instance_node.clone());

            return Some(Expression::StructInstance(struct_instance_node));
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
                            let var = VarMetadataNode {
                                name: field.value,
                                type_name: type_name.value,
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
        let mut ex = expr.clone();

        let ops = "+-*/=<>!";
        while let Some(potential_op) = self.lexer.peek_char() {
            if !ops.contains(potential_op) {
                break;
            }

            let op_token = self.lexer.next().unwrap();
            let op = self.token_type_to_binary_op(op_token.kind);

            if let BinaryOp::Inc | BinaryOp::Dec = op {
                if let Some(Expression::Variable(..)) = expr.clone() {
                    let rhs = Box::new(Expression::Literal(
                        Token::from(
                            TokenType::Literal(LiteralType::Number),
                            String::from("1"),
                            self.lexer.get_cursor_pos(),
                        ),
                        LiteralType::Number,
                    ));

                    if let Some(lhs) = ex {
                        let binary_op_node = BinaryOpNode {
                            lhs: Box::new(lhs),
                            op,
                            rhs,
                        };

                        ex = Some(Expression::BinaryOp(binary_op_node));
                    }
                } else {
                    let rhs = Box::new(Expression::Literal(
                        Token::from(
                            TokenType::Literal(LiteralType::Number),
                            String::from("1"),
                            self.lexer.get_cursor_pos(),
                        ),
                        LiteralType::Number,
                    ));

                    if let Some(lhs) = ex {
                        let binary_op_node = BinaryOpNode {
                            lhs: Box::new(lhs),
                            op,
                            rhs,
                        };

                        ex = Some(Expression::BinaryOp(binary_op_node));
                    }
                }
            } else {
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
                            let binary_op_node = BinaryOpNode {
                                lhs: Box::new(lhs),
                                op,
                                rhs,
                            };

                            ex = Some(Expression::BinaryOp(binary_op_node));
                        }
                    }
                }
            }
        }

        ex
    }

    fn default_initialize_value(&mut self, type_name: String) -> Expression {
        if let Some(struct_def_node) = self
            .structs
            .clone()
            .iter()
            .find(|&s| s.type_name == type_name)
        {
            return self.default_initialize_struct(struct_def_node);
        }

        let kind;
        let token = match type_name.as_str() {
            "char" => {
                kind = LiteralType::Char;
                Token::from(
                    TokenType::Literal(kind),
                    String::from(""),
                    self.lexer.get_cursor_pos(),
                )
            }
            "bool" => {
                kind = LiteralType::Bool;
                Token::from(
                    TokenType::Literal(kind),
                    String::from("false"),
                    self.lexer.get_cursor_pos(),
                )
            }
            "i32" => {
                kind = LiteralType::Number;
                Token::from(
                    TokenType::Literal(kind),
                    String::from("0"),
                    self.lexer.get_cursor_pos(),
                )
            }
            "f32" => {
                kind = LiteralType::Float;
                Token::from(
                    TokenType::Literal(kind),
                    String::from("0.0"),
                    self.lexer.get_cursor_pos(),
                )
            }
            "String" => {
                kind = LiteralType::String;
                Token::from(
                    TokenType::Literal(kind),
                    String::from(""),
                    self.lexer.get_cursor_pos(),
                )
            }
            _ => panic!("unimplemented literal type"),
        };

        Expression::Literal(token, kind)
    }

    fn default_initialize_struct(&mut self, struct_def_node: &StructDefNode) -> Expression {
        let mut fields = Vec::new();

        for field in struct_def_node.fields.clone().iter() {
            let field_name = field.name.clone();
            let type_name = field.type_name.clone();

            let value = self.default_initialize_value(type_name.clone());
            let variable = self.make_variable(field_name, type_name, Box::new(value));

            fields.push(variable);
        }

        let struct_instance_node = StructInstanceNode {
            struct_def: struct_def_node.clone(),
            fields,
        };

        Expression::StructInstance(struct_instance_node)
    }

    fn make_variable(
        &self,
        name: String,
        type_name: String,
        value: Box<Expression>,
    ) -> VariableNode {
        VariableNode {
            metadata: VarMetadataNode { name, type_name },
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

    fn token_type_to_binary_op(&self, kind: TokenType) -> BinaryOp {
        type TT = TokenType;
        match kind {
            TT::Inc => BinaryOp::Inc,
            TT::Dec => BinaryOp::Dec,
            TT::Add => BinaryOp::Add,
            TT::AddAssign => BinaryOp::AddAssign,
            TT::Sub => BinaryOp::Sub,
            TT::SubAssign => BinaryOp::SubAssign,
            TT::Mul => BinaryOp::Mul,
            TT::MulAssign => BinaryOp::SubAssign,
            TT::Div => BinaryOp::Div,
            TT::DivAssign => BinaryOp::SubAssign,
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
