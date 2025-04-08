use super::absyn::TypeExpression;

#[derive(Debug)]
pub struct ArrayTypeExpression {
    pub array_size: usize,
    pub base_type: TypeExpression,
}
