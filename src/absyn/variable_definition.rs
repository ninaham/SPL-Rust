use super::absyn::TypeExpression;

pub struct VariableDefinition<'a> {
    pub name: String,
    pub type_expression: TypeExpression<'a>,
}
