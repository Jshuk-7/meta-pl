use crate::{expression::Expression, token::LiteralType};

#[derive(Debug, Clone)]
pub enum BinaryOp {
    None,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,
    Neg,
}

#[derive(Debug, Clone)]
pub struct IfNode {
    pub value: Box<Expression>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct LetNode {
    pub name: String,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct AssignNode {
    pub value: VariableNode,
    pub new_value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ReturnNode {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct VarDefNode {
    pub name: String,
    pub kind: LiteralType,
}

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub metadata: VarDefNode,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ProcDefNode {
    pub name: String,
    pub return_type: Option<String>,
    pub args: Vec<VarDefNode>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct StructDefNode {
    pub type_name: String,
    pub fields: Vec<VarDefNode>,
}

#[derive(Debug, Clone)]
pub struct FunCallNode {
    pub proc_def: ProcDefNode,
    pub args: Vec<VariableNode>,
}

#[derive(Debug, Clone)]
pub struct BinaryOpNode {
    pub lhs: Box<Expression>,
    pub op: BinaryOp,
    pub rhs: Box<Expression>,
}
