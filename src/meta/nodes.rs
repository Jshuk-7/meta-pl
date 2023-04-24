use crate::expression::Expression;

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
pub struct WhileNode {
    pub value: Box<Expression>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ForNode {
    pub counter: VariableNode,
    pub range: Box<Expression>,
    pub statements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct RangeNode {
    pub start: Box<Expression>,
    pub end: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct LetNode {
    pub name: String,
    pub type_name: String,
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
    pub type_name: String,
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
pub struct ImplNode {
    pub procedures: Vec<Expression>,
    pub struct_def: StructDefNode,
}

#[derive(Debug, Clone)]
pub struct ImplFunCallNode {
    pub impl_node: ImplNode,
    pub fun_call_node: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct StructInstanceNode {
    pub struct_def: StructDefNode,
    pub fields: Vec<VariableNode>,
}

#[derive(Debug, Clone)]
pub struct FieldAssignNode {
    pub struct_instance: VariableNode,
    pub field: VariableNode,
    pub new_value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct FieldAccessNode {
    pub struct_instance: VariableNode,
    pub field: VariableNode,
}

#[derive(Debug, Clone)]
pub struct BinaryOpNode {
    pub lhs: Box<Expression>,
    pub op: BinaryOp,
    pub rhs: Box<Expression>,
}
