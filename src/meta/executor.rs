use std::path::Path;

use crate::{
    expression::Expression,
    nodes::{LetNode, ProcDefNode, VarDefNode, VariableNode},
    parser::{Parser, Program},
    token::LiteralType,
};

pub struct Executor {}

impl Executor {
    pub fn run<P: AsRef<Path> + Clone>(path: P) {
        let mut variables = Vec::<VariableNode>::new();

        if let Ok(mut parser) = Parser::from_file(path) {
            if let Ok(program) = parser.parse_program() {
                if let Some(main_proc) = Executor::find_startup_proc(program) {
                    Executor::execute_process(main_proc, &mut variables);
                }
            }
        }
    }

    fn find_startup_proc(program: Program) -> Option<ProcDefNode> {
        if let Expression::ProcDef(proc_def_node) = program
            .iter()
            .find(move |&expr| {
                if let Expression::ProcDef(ProcDefNode { name, .. }) = expr {
                    name == "main"
                } else {
                    false
                }
            })
            .unwrap()
            .clone()
        {
            return Some(proc_def_node);
        }

        None
    }

    fn execute_process(proc_def: ProcDefNode, variables: &mut Vec<VariableNode>) {
        for statement in proc_def.statements.iter() {
            Executor::execute_statement(statement, variables);
        }
    }

    fn execute_statement(statement: &Expression, variables: &mut Vec<VariableNode>) {
        match statement {
            Expression::LetStatement(let_node) => {
                let var = VariableNode {
                    metadata: VarDefNode {
                        name: let_node.name.clone(),
                        kind: LiteralType::Number,
                    },
                    value: let_node.value.clone(),
                };

                variables.push(var);
            }
            Expression::AssignStatement(assign_node) => {
                let variable = variables
                    .iter_mut()
                    .find(|v| *v.metadata.name == assign_node.value.metadata.name)
                    .unwrap();

                variable.value = assign_node.new_value.clone();
            }
            Expression::ReturnStatement(_) => todo!(),
            Expression::IfStatement(_) => {
                
            },
            Expression::ProcDef(_) => todo!(),
            Expression::FunCall(fun_call_node) => {
                Executor::execute_process(fun_call_node.proc_def.clone(), variables)
            }
            Expression::Variable(_) => todo!(),
            Expression::StructDef(_) => todo!(),
            Expression::BinaryOp(_) => todo!(),
            Expression::Literal(_, _) => todo!(),
        }
    }
}
