use std::fmt::{Display, Write};

use crate::token::{LiteralType, Token};

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub var: Var,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
    LetStatement {
        name: String,
        value: Box<Expression>,
    },

    AssignStatement {
        name: String,
        new_value: Box<Expression>,
    },

    ProcDef {
        name: String,
        return_type: Option<String>,
        return_value: Option<Box<Expression>>,
        args: Vec<Var>,
        statements: Vec<Expression>,
    },

    Value(Variable),
    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
    Literal(Token, LiteralType),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::LetStatement { name, value } => {
                f.write_fmt(format_args!("Let({name} = {value})"))
            }
            Expression::AssignStatement { name, new_value } => {
                f.write_fmt(format_args!("Assign({name} = {new_value})"))
            }
            Expression::ProcDef {
                name,
                statements,
                args,
                return_type,
                return_value,
            } => {
                let mut arguments = String::new();
                if !args.is_empty() {
                    arguments.push('\n');
                }
                for arg in args.iter() {
                    arguments
                        .write_fmt(format_args!("\t\t{}: {},\n", arg.name, arg.kind))
                        .unwrap();
                }
                if !args.is_empty() {
                    arguments.push('\t');
                }

                let mut content = String::new();
                if !statements.is_empty() {
                    content.push('\n');
                }
                for statement in statements.iter() {
                    content
                        .write_fmt(format_args!("\t\t{statement}\n"))
                        .unwrap();
                }
                if !statements.is_empty() {
                    content.push('\t');
                }

                let mut return_type_str = String::from("None");
                if let Some(rt) = return_type {
                    return_type_str = rt.clone();
                }

                let mut return_value_str = String::from("None");
                if let Some(rv) = return_value {
                    return_value_str = format!("{rv}");
                }

                f.write_fmt(format_args!(
                    "ProcDef {name} {{
\treturn_type: {return_type_str}
\treturn_value: {return_value_str}
\targs: [{arguments}]
\tcontent: [{content}]\n}}\n"
                ))
            }
            Expression::Value(var) => f.write_fmt(format_args!(
                "{}({}, {})",
                var.var.name, var.value, var.var.kind
            )),
            Expression::BinaryOperation(lhs, op, rhs) => {
                f.write_fmt(format_args!("BinaryOperation({lhs}, {op:?}, {rhs})"))
            }
            Expression::Literal(token, _type) => {
                f.write_fmt(format_args!("Literal('{}', {_type:?})", token.value))
            }
        }
    }
}
