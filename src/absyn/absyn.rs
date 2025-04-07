use super::{
    array_access::ArrayAccess, array_type_expression::ArrayTypeExpression,
    assign_statement::AssignStatement, binary_expression::BinaryExpression,
    call_statement::CallStatement, if_statement::IfStatement,
    parameter_definition::ParameterDefinition, procedure_definition::ProcedureDefinition,
    type_definition::TypeDefinition, unary_expression::UnaryExpression,
    variable_definition::VariableDefinition, while_statement::WhileStatement,
};

enum Node {
    Program(Vec<Definition>),
    Variable(Variable),
    Expression(Expression),
    Statement(Statement),
    TypeExpression(TypeExpression),
    ParameterDefinition(ParameterDefinition),
    VariableDefinition(VariableDefinition),
}

enum Definition {
    ProcedureDefinition(ProcedureDefinition),
    TypeDefinition(TypeDefinition),
}

enum Variable {
    NamedVariable(String),
    ArrayAccess(ArrayAccess),
}

enum TypeExpression {
    ArrayTypeExpression(ArrayTypeExpression),
    NamedTypeExpression(String),
}

enum Expression {
    BinaryExpression(BinaryExpression),
    UnaryExpression(UnaryExpression),
    IntLiteral(i64),
    VariableExpression(Variable),
}

enum Statement {
    AssignStatement(AssignStatement),
    IfStatement(IfStatement),
    WhileStatement(WhileStatement),
    CallStatement(CallStatement),
    EmptyStatement,
    CompoundStatement(Vec<Statement>),
}
