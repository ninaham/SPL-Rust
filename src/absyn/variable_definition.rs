use super::absyn::TypeExpression;

pub struct VariableDefinition {
    pub name: String,
    pub type_expression: TypeExpression,
}
