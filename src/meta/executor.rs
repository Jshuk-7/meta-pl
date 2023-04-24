use std::path::Path;

use crate::{
    expression::Expression,
    nodes::{ProcDefNode, StructInstanceNode, VarMetadataNode, VariableNode},
    parser::{Parser, Program},
};

const ENTRY_POINT: &str = "main";

pub struct Executor {}

struct RuntimeVM {
    pub variables: Vec<VariableNode>,
    pub structs: Vec<StructInstanceNode>,
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
        let proc = program.iter().find(move |&expr| {
            if let Expression::ProcDef(ProcDefNode { name, .. }) = expr {
                return name == target;
            }

            false
        });

        if let Some(Expression::ProcDef(proc_def_node)) = proc {
            return Some(proc_def_node.clone());
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
            Expression::IfStatement(..) => {}
            Expression::WhileStatement(..) => {}
            Expression::ForLoop(..) => {}
            Expression::RangeStatement(..) => {}
            Expression::LetStatement(let_node) => {
                let metadata = VarMetadataNode {
                    name: let_node.name.clone(),
                    type_name: let_node.type_name.clone(),
                };

                let var = VariableNode {
                    metadata,
                    value: let_node.value.clone(),
                };

                if let Expression::StructInstance(_) = let_node.value.as_ref() {
                    Executor::execute_statement(let_node.value.as_ref(), memory);
                }

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
            Expression::ReturnStatement(..) => {}
            Expression::Variable(..) => {}
            Expression::ProcDef(..) => todo!(),
            Expression::FunCall(fun_call_node) => {
                Executor::execute_procedure(fun_call_node.proc_def.clone(), memory)
            }
            Expression::StructDef(..) => todo!(),
            Expression::ImplStatement(..) => todo!(),
            Expression::ImplFunCall(impl_fun_call_node) => {
                if let Expression::ProcDef(proc_def_node) =
                    impl_fun_call_node.fun_call_node.as_ref()
                {
                    Executor::execute_procedure(proc_def_node.clone(), memory)
                }
            }
            Expression::StructInstance(struct_instance_node) => {
                memory.structs.push(struct_instance_node.clone());
            }
            Expression::StructFieldAssign(field_assign_node) => {
                'outer: for (i, struct_instance) in memory.structs.clone().iter().enumerate() {
                    for (j, field) in struct_instance.fields.iter().enumerate() {
                        if field.metadata.name == field_assign_node.field.metadata.name {
                            memory.structs[i].fields[j].value = field_assign_node.new_value.clone();
                            break 'outer;
                        }
                    }
                }
            }
            Expression::StructFieldAccess(_) => {}
            Expression::BinaryOp(_) => todo!(),
            Expression::Literal(_, _) => todo!(),
        }

        None
    }
}
