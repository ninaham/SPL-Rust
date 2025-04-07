use super::{absyn::Statement, parameter_definition::ParameterDefinition, variable_definition::VariableDefinition};

pub struct ProcedureDefinition<'a> {
    pub name: String,
    pub parameters: Vec<ParameterDefinition<'a>>,
    pub body: Vec<Statement<'a>>,
    pub variales: Vec<VariableDefinition<'a>>,
}
