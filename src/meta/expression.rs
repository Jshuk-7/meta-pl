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
pub struct VarDef {
    pub name: String,
    pub kind: LiteralType,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub var: VarDef,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ProcDef {
    pub name: String,
    pub return_type: Option<String>,
    pub return_value: Option<Box<Expression>>,
    pub args: Vec<VarDef>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    LetStatement {
        name: String,
        value: Box<Expression>,
    },

    AssignStatement {
        value: Variable,
        new_value: Box<Expression>,
    },

    FunCall {
        proc_def: ProcDef,
        args: Vec<Variable>,
    },

    ProcDef(ProcDef),
    Variable(Variable),
    BinaryOperation(Box<Expression>, BinaryOp, Box<Expression>),
    Literal(Token, LiteralType),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::LetStatement { name, value } => {
                f.write_fmt(format_args!("Let('{name}' = {value})"))
            }
            Expression::AssignStatement { value, new_value } => {
                let name = value.var.name.clone();
                f.write_fmt(format_args!("Assign('{name}' = {new_value})"))
            }
            Expression::FunCall { proc_def, args } => {
                let mut arguments = String::new();

                if !args.is_empty() {
                    arguments.push('\n');
                }
                for arg in args.iter() {
                    arguments
                        .write_fmt(format_args!("\t\t\t{}: {}\n", arg.var.name, arg.value))
                        .unwrap();
                }
                if !args.is_empty() {
                    arguments.push_str("\t\t");
                }

                let name = proc_def.name.clone();
                f.write_fmt(format_args!("FunCall('{name}', args: [{arguments}])"))
            }
            Expression::ProcDef(proc_def) => {
                let mut arguments = String::new();
                if !proc_def.args.is_empty() {
                    arguments.push('\n');
                }
                for arg in proc_def.args.iter() {
                    arguments
                        .write_fmt(format_args!("\t\t{}: {:?},\n", arg.name, arg.kind))
                        .unwrap();
                }
                if !proc_def.args.is_empty() {
                    arguments.push('\t');
                }

                let mut content = String::new();
                if !proc_def.statements.is_empty() {
                    content.push('\n');
                }
                for statement in proc_def.statements.iter() {
                    content
                        .write_fmt(format_args!("\t\t{statement}\n"))
                        .unwrap();
                }
                if !proc_def.statements.is_empty() {
                    content.push('\t');
                }

                let mut return_type_str = String::from("None");
                if let Some(rt) = proc_def.return_type.clone() {
                    return_type_str = rt;
                }

                let mut return_value_str = String::from("None");
                if let Some(rv) = proc_def.return_value.clone() {
                    return_value_str = format!("{rv}");
                }

                f.write_fmt(format_args!(
                    "ProcDef {} {{
\treturn_type: {return_type_str}
\treturn_value: {return_value_str}
\targs: [{arguments}]
\tcontent: [{content}]\n}}\n",
                    proc_def.name
                ))
            }
            Expression::Variable(var) => {
                f.write_fmt(format_args!("Variable({}: {})", var.var.name, var.value,))
            }
            Expression::BinaryOperation(lhs, op, rhs) => {
                f.write_fmt(format_args!("BinaryOperation({lhs}, {op:?}, {rhs})"))
            }
            Expression::Literal(token, _type) => {
                f.write_fmt(format_args!("Literal('{}', {_type:?})", token.value))
            }
        }
    }
}
