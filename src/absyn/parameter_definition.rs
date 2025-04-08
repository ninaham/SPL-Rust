use super::absyn::TypeExpression;

pub struct ParameterDefinition {
    pub name: String,
    pub type_expression: TypeExpression,
    pub is_reference: bool,
}
