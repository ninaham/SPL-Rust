use std::collections::LinkedList;

use super::{
    array_access::ArrayAccess, array_type_expression::ArrayTypeExpression,
    assign_statement::AssignStatement, binary_expression::BinaryExpression,
    call_statement::CallStatement, if_statement::IfStatement,
    procedure_definition::ProcedureDefinition, type_definition::TypeDefinition,
    unary_expression::UnaryExpression, while_statement::WhileStatement,
};

#[derive(Debug)]
pub struct Program {
    pub definitions: LinkedList<Box<Definition>>,
}

#[derive(Debug, Clone)]
pub enum Definition {
    ProcedureDefinition(Box<ProcedureDefinition>),
    TypeDefinition(Box<TypeDefinition>),
}

#[derive(Debug, Clone)]
pub enum Variable {
    NamedVariable(String),
    ArrayAccess(Box<ArrayAccess>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpression {
    ArrayTypeExpression(Box<ArrayTypeExpression>),
    NamedTypeExpression(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    BinaryExpression(Box<BinaryExpression>),
    UnaryExpression(Box<UnaryExpression>),
    IntLiteral(i64),
    VariableExpression(Box<Variable>),
}

#[derive(Debug, Clone)]
pub enum Statement {
    AssignStatement(Box<AssignStatement>),
    IfStatement(Box<IfStatement>),
    WhileStatement(Box<WhileStatement>),
    CallStatement(Box<CallStatement>),
    EmptyStatement,
    CompoundStatement(LinkedList<Box<Statement>>),
}
