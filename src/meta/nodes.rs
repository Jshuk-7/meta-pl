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
    pub kind: LiteralType,
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
pub struct VarMetadataNode {
    pub name: String,
    pub kind: LiteralType,
}

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub metadata: VarMetadataNode,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ProcDefNode {
    pub name: String,
    pub return_type: Option<String>,
    pub args: Vec<VarMetadataNode>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunCallNode {
    pub proc_def: ProcDefNode,
    pub args: Vec<VariableNode>,
}

#[derive(Debug, Clone)]
pub struct StructDefNode {
    pub type_name: String,
    pub fields: Vec<VarMetadataNode>,
}

#[derive(Debug, Clone)]
pub struct StructInstanceNode {
    pub struct_def: StructDefNode,
    pub fields: Vec<VariableNode>,
}

#[derive(Debug, Clone)]
pub struct BinaryOpNode {
    pub lhs: Box<Expression>,
    pub op: BinaryOp,
    pub rhs: Box<Expression>,
}
