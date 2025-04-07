use super::absyn::TypeExpression;

pub struct ParameterDefinition<'a> {
    pub name: String,
    pub type_expression: TypeExpression<'a>,
    pub is_reference: bool,
}
