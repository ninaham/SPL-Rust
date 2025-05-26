use super::absyn::Expression;

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub operator: Operator,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Equ,
    Neq,
    Lst,
    Lse,
    Grt,
    Gre,
}
