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
pub struct Argument {
    pub name: String,
    pub _type: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
    LetStatement {
        name: String,
        value: Box<Expression>,
    },

    ProcDef {
        name: String,
        return_type: Option<String>,
        return_value: Option<Box<Expression>>,
        args: Vec<Argument>,
        statements: Vec<Expression>,
    },

    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
    Literal(Token, LiteralType),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::LetStatement { name, value } => {
                f.write_fmt(format_args!("Let ({name} = {value})"))
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
                        .write_fmt(format_args!("\t{}: {},\n", arg.name, arg._type))
                        .unwrap();
                }

                let mut content = String::new();
                if !statements.is_empty() {
                    content.push('\n');
                }
                for statement in statements.iter() {
                    content.write_fmt(format_args!("\t{statement}\n")).unwrap();
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
                    "ProcDef {name}
return_type: {return_type_str}
return_value: {return_value_str}
args: [{arguments}]
content: [{content}]"
                ))
            }
            Expression::BinaryOperation(lhs, op, rhs) => {
                f.write_fmt(format_args!("BinaryOperation ({lhs}, {op:?}, {rhs})"))
            }
            Expression::Literal(token, _type) => {
                f.write_fmt(format_args!("Literal ('{}', {_type:?})", token.value))
            }
        }
    }
}
