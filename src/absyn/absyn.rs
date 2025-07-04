#![expect(clippy::enum_variant_names, clippy::linkedlist)]

use std::collections::LinkedList;

use super::{
    array_access::ArrayAccess, array_type_expression::ArrayTypeExpression,
    assign_statement::AssignStatement, binary_expression::BinaryExpression,
    call_statement::CallStatement, if_statement::IfStatement,
    procedure_definition::ProcedureDefinition, type_definition::TypeDefinition,
    unary_expression::UnaryExpression, while_statement::WhileStatement,
};

/// Represents a complete program consisting of multiple definitions.
#[derive(Debug, Clone)]
pub struct Program {
    /// List of definitions (procedures or types) contained in the program.
    pub definitions: LinkedList<Box<Definition>>,
}

/// Represents either a procedure or type definition in the program.
#[derive(Debug, Clone)]
pub enum Definition {
    /// A procedure definition (function or method).
    ProcedureDefinition(Box<ProcedureDefinition>),
    /// A user-defined type definition.
    TypeDefinition(Box<TypeDefinition>),
}

/// Represents a variable, which can be a named variable or an array access.
#[derive(Debug, Clone)]
pub enum Variable {
    /// A simple named variable identified by a string.
    NamedVariable(String),
    /// Access to an array element via an index expression.
    ArrayAccess(Box<ArrayAccess>),
}

/// Represents the type of an expression, either a named type or an array type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpression {
    /// An array type expression (e.g., int[]).
    ArrayTypeExpression(Box<ArrayTypeExpression>),
    /// A named type expression (e.g., int, float, custom types).
    NamedTypeExpression(String),
}

/// Represents expressions in the language.
#[derive(Debug, Clone)]
pub enum Expression {
    /// A binary expression with two operands and an operator.
    BinaryExpression(Box<BinaryExpression>),
    /// A unary expression with one operand and an operator.
    UnaryExpression(Box<UnaryExpression>),
    /// An integer literal value.
    IntLiteral(i32),
    /// A variable expression, referring to a variable.
    VariableExpression(Box<Variable>),
}

/// Represents statements that can appear in the program.
#[derive(Debug, Clone)]
pub enum Statement {
    /// Assignment statement (e.g., `x = 5`).
    AssignStatement(Box<AssignStatement>),
    /// Conditional if-statement.
    IfStatement(Box<IfStatement>),
    /// While loop statement.
    WhileStatement(Box<WhileStatement>),
    /// Procedure or function call statement.
    CallStatement(Box<CallStatement>),
    /// An empty statement (no operation).
    EmptyStatement,
    /// A compound statement containing a list of statements.
    CompoundStatement(LinkedList<Box<Statement>>),
}
