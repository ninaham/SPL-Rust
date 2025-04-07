use super::absyn::Expression;

pub struct UnaryExpression<'a> {
    pub operator: UnaryOperator,
    pub operand: &'a Expression<'a>,
}

pub enum UnaryOperator {
    Minus,
}
