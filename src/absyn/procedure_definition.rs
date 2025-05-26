#![expect(clippy::linkedlist)]

use std::collections::LinkedList;

use super::{
    absyn::Statement, parameter_definition::ParameterDefinition,
    variable_definition::VariableDefinition,
};

#[derive(Debug, Clone)]
pub struct ProcedureDefinition {
    pub name: String,
    pub parameters: LinkedList<ParameterDefinition>,
    pub body: LinkedList<Statement>,
    pub variales: LinkedList<VariableDefinition>,
}
