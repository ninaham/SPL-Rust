use super::absyn::TypeExpression;

pub struct ArrayTypeExpression<'a> {
    pub array_size: usize,
    pub base_type: &'a TypeExpression<'a>
}
