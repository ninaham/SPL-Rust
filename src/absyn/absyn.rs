use std::collections::LinkedList;

use super::{
    array_access::ArrayAccess, array_type_expression::ArrayTypeExpression,
    assign_statement::AssignStatement, binary_expression::BinaryExpression,
    call_statement::CallStatement, if_statement::IfStatement,
    parameter_definition::ParameterDefinition, procedure_definition::ProcedureDefinition,
    type_definition::TypeDefinition, unary_expression::UnaryExpression,
    variable_definition::VariableDefinition, while_statement::WhileStatement,
};

pub enum Node {
    Program(LinkedList<Box<Definition>>),
    Variable(Box<Variable>),
    Expression(Box<Expression>),
    Statement(Box<Statement>),
    TypeExpression(Box<TypeExpression>),
    ParameterDefinition(Box<ParameterDefinition>),
    VariableDefinition(Box<VariableDefinition>),
}

pub enum Definition {
    ProcedureDefinition(Box<ProcedureDefinition>),
    TypeDefinition(Box<TypeDefinition>),
}

pub enum Variable {
    NamedVariable(String),
    ArrayAccess(Box<ArrayAccess>),
}

pub enum TypeExpression {
    ArrayTypeExpression(Box<ArrayTypeExpression>),
    NamedTypeExpression(String),
}

pub enum Expression {
    BinaryExpression(Box<BinaryExpression>),
    UnaryExpression(Box<UnaryExpression>),
    IntLiteral(i64),
    VariableExpression(Box<Variable>),
}

pub enum Statement {
    AssignStatement(Box<AssignStatement>),
    IfStatement(Box<IfStatement>),
    WhileStatement(Box<WhileStatement>),
    CallStatement(Box<CallStatement>),
    EmptyStatement,
    CompoundStatement(LinkedList<Box<Statement>>),
}
