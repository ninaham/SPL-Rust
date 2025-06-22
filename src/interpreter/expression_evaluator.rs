use crate::{
    absyn::{
        absyn::{Expression, Variable},
        binary_expression::{self, BinaryExpression},
        unary_expression::{self, UnaryExpression},
    },
    interpreter::{environment::Environment, value::Value},
};

pub fn eval_expression(expression: &Expression, env: &Environment) -> Value {
    match expression {
        Expression::BinaryExpression(binary_expression) => eval_binary(binary_expression, env),
        Expression::UnaryExpression(unary_expression) => eval_unary(unary_expression, env),
        Expression::IntLiteral(i) => Value::Int(*i),
        Expression::VariableExpression(variable) => eval_var(variable, env),
    }
}

pub fn eval_var(variable: &Variable, env: &Environment) -> Value {
    match variable {
        Variable::NamedVariable(v) => env.vars.get(v).unwrap().clone(),
        Variable::ArrayAccess(array_access) => todo!(),
    }
}

pub fn eval_binary(binary_expression: &BinaryExpression, env: &Environment) -> Value {
    let op1 = eval_expression(&binary_expression.left, env);
    let op2 = eval_expression(&binary_expression.right, env);

    match binary_expression.operator {
        binary_expression::Operator::Add => op1 + op2,
        binary_expression::Operator::Sub => op1 - op2,
        binary_expression::Operator::Mul => op1 * op2,
        binary_expression::Operator::Div => op1 / op2,
        binary_expression::Operator::Equ => Value::Bool(op1 == op2),
        binary_expression::Operator::Neq => Value::Bool(op1 != op2),
        binary_expression::Operator::Lst => Value::Bool(op1 < op2),
        binary_expression::Operator::Lse => Value::Bool(op1 <= op2),
        binary_expression::Operator::Grt => Value::Bool(op1 > op2),
        binary_expression::Operator::Gre => Value::Bool(op1 >= op2),
    }
}

pub fn eval_unary(unary: &UnaryExpression, env: &Environment) -> Value {
    let op = eval_expression(&unary.operand, env);

    match unary.operator {
        unary_expression::UnaryOperator::Minus => -op,
    }
}
