use super::absyn::Expression;

#[derive(Debug)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
}
