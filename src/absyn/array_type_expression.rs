use super::absyn::TypeExpression;

pub struct ArrayTypeExpression {
    pub array_size: usize,
    pub base_type: TypeExpression,
}
