use super::{
    array_access::ArrayAccess, array_type_expression::ArrayTypeExpression,
    assign_statement::AssignStatement, binary_expression::BinaryExpression,
    call_statement::CallStatement, if_statement::IfStatement,
    parameter_definition::ParameterDefinition, procedure_definition::ProcedureDefinition,
    type_definition::TypeDefinition, unary_expression::UnaryExpression,
    variable_definition::VariableDefinition, while_statement::WhileStatement,
};

pub enum Node<'a> {
    Program(Vec<Definition<'a>>),
    Variable(Variable<'a>),
    Expression(Expression<'a>),
    Statement(Statement<'a>),
    TypeExpression(TypeExpression<'a>),
    ParameterDefinition(ParameterDefinition<'a>),
    VariableDefinition(VariableDefinition<'a>),
}

pub enum Definition<'a> {
    ProcedureDefinition(ProcedureDefinition<'a>),
    TypeDefinition(TypeDefinition<'a>),
}

pub enum Variable<'a> {
    NamedVariable(String),
    ArrayAccess(ArrayAccess<'a>),
}

pub enum TypeExpression<'a> {
    ArrayTypeExpression(ArrayTypeExpression<'a>),
    NamedTypeExpression(String),
}

pub enum Expression<'a> {
    BinaryExpression(BinaryExpression<'a>),
    UnaryExpression(UnaryExpression<'a>),
    IntLiteral(i64),
    VariableExpression(Variable<'a>),
}

pub enum Statement<'a> {
    AssignStatement(AssignStatement<'a>),
    IfStatement(IfStatement<'a>),
    WhileStatement(WhileStatement<'a>),
    CallStatement(CallStatement<'a>),
    EmptyStatement,
    CompoundStatement(Vec<Statement<'a>>),
}
