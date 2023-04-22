use std::fmt::{Display, Write};

use crate::{
    nodes::{
        AssignNode, BinaryOpNode, FunCallNode, IfNode, LetNode, ProcDefNode, ReturnNode,
        StructDefNode, StructInstanceNode, VariableNode,
    },
    token::{LiteralType, Token},
};

#[derive(Debug, Clone)]
pub enum Expression {
    IfStatement(IfNode),
    LetStatement(LetNode),
    AssignStatement(AssignNode),
    ReturnStatement(ReturnNode),
    Variable(VariableNode),
    ProcDef(ProcDefNode),
    FunCall(FunCallNode),
    StructDef(StructDefNode),
    StructInstance(StructInstanceNode),
    BinaryOp(BinaryOpNode),
    Literal(Token, LiteralType),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::IfStatement(if_node) => {
                let mut statements = String::new();
                if !if_node.statements.is_empty() {
                    statements.push('\n');
                }
                for statement in if_node.statements.iter() {
                    statements
                        .write_fmt(format_args!("\t\t\t{statement}\n"))
                        .unwrap()
                }
                if !if_node.statements.is_empty() {
                    statements.push_str("\t\t");
                }

                f.write_fmt(format_args!("If({}: [{statements}])", if_node.value))
            }
            Expression::LetStatement(let_node) => {
                f.write_fmt(format_args!("Let('{}': {})", let_node.name, let_node.value))
            }
            Expression::AssignStatement(assign_node) => {
                let name = assign_node.value.metadata.name.clone();
                f.write_fmt(format_args!("Assign('{name}': {})", assign_node.new_value))
            }
            Expression::ReturnStatement(return_node) => {
                f.write_fmt(format_args!("Return({})", return_node.value))
            }
            Expression::Variable(var) => f.write_fmt(format_args!(
                "Variable('{}': {})",
                var.metadata.name, var.value,
            )),
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

                f.write_fmt(format_args!(
                    "ProcDef('{}':
\treturn_type: {return_type_str}
\targs: [{arguments}]
\tcontent: [{content}]\n)\n",
                    proc_def.name
                ))
            }
            Expression::FunCall(fun_call_node) => {
                let mut arguments = String::new();

                if !fun_call_node.args.is_empty() {
                    arguments.push('\n');
                }
                for arg in fun_call_node.args.iter() {
                    arguments
                        .write_fmt(format_args!("\t\t\t{}: {}\n", arg.metadata.name, arg.value))
                        .unwrap();
                }
                if !fun_call_node.args.is_empty() {
                    arguments.push_str("\t\t");
                }

                let name = fun_call_node.proc_def.name.clone();
                f.write_fmt(format_args!("FunCall('{name}': args: [{arguments}])"))
            }
            Expression::StructDef(struct_def) => {
                let mut fields = String::new();
                if !struct_def.fields.is_empty() {
                    fields.push('\n');
                }
                for field in struct_def.fields.iter() {
                    fields
                        .write_fmt(format_args!("\t{}: {:?},\n", field.name, field.kind))
                        .unwrap();
                }

                f.write_fmt(format_args!(
                    "StructDef('{}': fields: [{fields}])\n",
                    struct_def.type_name
                ))
            }
            Expression::StructInstance(struct_instance_node) => {
                let mut fields = String::new();
                if !struct_instance_node.fields.is_empty() {
                    fields.push('\n');
                }
                for field in struct_instance_node.fields.iter() {
                    fields
                        .write_fmt(format_args!(
                            "\t\t\t{}: {},\n",
                            field.metadata.name, field.value
                        ))
                        .unwrap();
                }
                if !struct_instance_node.fields.is_empty() {
                    fields.push_str("\t\t");
                }

                f.write_fmt(format_args!(
                    "Struct('{}': fields: [{fields}])",
                    struct_instance_node.struct_def.type_name
                ))
            }
            Expression::BinaryOp(binary_op_node) => f.write_fmt(format_args!(
                "BinaryOp({}, {:?}, {})",
                binary_op_node.lhs, binary_op_node.op, binary_op_node.rhs
            )),
            Expression::Literal(token, _type) => {
                f.write_fmt(format_args!("Literal('{}': {_type:?})", token.value))
            }
        }
    }
}
