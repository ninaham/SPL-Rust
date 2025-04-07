use super::absyn::Expression;

pub struct BinaryExpression<'a> {
    pub operator: Operator,
    pub left: &'a Expression<'a>,
    pub right: &'a Expression<'a>,
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
