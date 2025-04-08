use super::{
    absyn::Statement, parameter_definition::ParameterDefinition,
    variable_definition::VariableDefinition,
};

pub struct ProcedureDefinition {
    pub name: String,
    pub parameters: Vec<ParameterDefinition>,
    pub body: Vec<Statement>,
    pub variales: Vec<VariableDefinition>,
}
