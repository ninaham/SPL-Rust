use super::absyn::Expression;

pub struct BinaryExpression {
    pub operator: Operator,
    pub left: Expression,
    pub right: Expression,
}

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
