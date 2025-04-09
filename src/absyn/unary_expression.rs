use super::absyn::Expression;

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Minus,
}
