use super::absyn::TypeExpression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayTypeExpression {
    pub array_size: usize,
    pub base_type: TypeExpression,
}
