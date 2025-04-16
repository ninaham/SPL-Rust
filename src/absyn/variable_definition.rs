use super::absyn::TypeExpression;

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub name: String,
    pub type_expression: TypeExpression,
}
