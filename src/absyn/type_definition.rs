use super::absyn::TypeExpression;

#[derive(Debug)]
pub struct TypeDefinition {
    pub name: String,
    pub type_expression: TypeExpression,
}
