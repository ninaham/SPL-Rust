use super::absyn::Expression;

#[derive(Debug)]
pub struct BinaryExpression {
    pub operator: Operator,
    pub left: Expression,
    pub right: Expression,
}

#[derive(Debug)]
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
