use super::absyn::Expression;

pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub operand: Expression,
}

pub enum UnaryOperator {
    Minus,
}
