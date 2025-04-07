use super::absyn::TypeExpression;

pub struct TypeDefinition<'a> {
    pub name: String,
    pub type_expression: TypeExpression<'a>,
}
