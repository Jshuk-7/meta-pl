use std::path::Path;

use crate::{
    expression::Expression,
    nodes::{ProcDefNode, StructDefNode, VarMetadataNode, VariableNode},
    parser::{Parser, Program},
};

const ENTRY_POINT: &str = "main";

pub struct Executor {}

struct RuntimeVM {
    pub variables: Vec<VariableNode>,
    pub structs: Vec<StructDefNode>,
}

impl RuntimeVM {
    fn new() -> Self {
        Self {
            variables: Vec::new(),
            structs: Vec::new(),
        }
    }
}

impl Executor {
    pub fn run<P: AsRef<Path> + Clone>(path: P) {
        let mut memory = RuntimeVM::new();

        if let Ok(mut parser) = Parser::from_file(path) {
            if let Ok(program) = parser.parse_program() {
                if let Some(main_proc) = Executor::find_startup_proc(program, ENTRY_POINT) {
                    Executor::execute_procedure(main_proc, &mut memory);
                }
            }
        }
    }

    fn find_startup_proc(program: Program, target: &str) -> Option<ProcDefNode> {
        if let Expression::ProcDef(proc_def_node) = program
            .iter()
            .find(move |&expr| {
                if let Expression::ProcDef(ProcDefNode { name, .. }) = expr {
                    name == target
                } else {
                    false
                }
            })
            .unwrap()
            .clone()
        {
            return Some(proc_def_node);
        }

        println!("Error: failed to find entry point '{target}'");
        None
    }

    fn execute_procedure(proc_def: ProcDefNode, memory: &mut RuntimeVM) {
        for statement in proc_def.statements.iter() {
            Executor::execute_statement(statement, memory);
        }
    }

    fn execute_statement(statement: &Expression, memory: &mut RuntimeVM) -> Option<Expression> {
        match statement {
            Expression::IfStatement(_) => {}
            Expression::LetStatement(let_node) => {
                let metadata = VarMetadataNode {
                    name: let_node.name.clone(),
                    kind: let_node.kind,
                };

                let var = VariableNode {
                    metadata,
                    value: let_node.value.clone(),
                };

                memory.variables.push(var);
            }
            Expression::AssignStatement(assign_node) => {
                let variable = memory
                    .variables
                    .iter_mut()
                    .find(|v| *v.metadata.name == assign_node.value.metadata.name)
                    .unwrap();

                variable.value = assign_node.new_value.clone();
            }
            Expression::ReturnStatement(_) => {}
            Expression::Variable(_) => todo!(),
            Expression::ProcDef(_) => todo!(),
            Expression::FunCall(fun_call_node) => {
                Executor::execute_procedure(fun_call_node.proc_def.clone(), memory)
            }
            Expression::StructDef(_) => todo!(),
            Expression::StructInstance(struct_instance_node) => {}
            Expression::BinaryOp(_) => todo!(),
            Expression::Literal(_, _) => todo!(),
        }

        None
    }
}
