use super::absyn::TypeExpression;

#[derive(Debug)]
pub struct VariableDefinition {
    pub name: String,
    pub type_expression: TypeExpression,
}
