use super::absyn::TypeExpression;

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub type_expression: TypeExpression,
}
