use std::rc::Rc;

use crate::{
    absyn::{
        absyn::{Expression, Variable},
        binary_expression::{self, BinaryExpression},
        unary_expression::{self, UnaryExpression},
    },
    interpreter::{environment::Environment, value::Value},
};

pub fn eval_expression<'a>(expression: &Expression, env: Rc<Environment<'a>>) -> Value<'a> {
    match expression {
        Expression::BinaryExpression(binary_expression) => eval_binary(binary_expression, env),
        Expression::UnaryExpression(unary_expression) => eval_unary(unary_expression, env),
        Expression::IntLiteral(i) => Value::Int(*i),
        Expression::VariableExpression(variable) => eval_var(variable, env),
    }
}

pub fn eval_var<'a>(variable: &Variable, env: Rc<Environment<'a>>) -> Value<'a> {
    match variable {
        Variable::NamedVariable(v) => env.get(v).unwrap(),
        Variable::ArrayAccess(array_access) => {
            let Value::Int(index) = eval_expression(&array_access.index, env.clone()) else {
                unreachable!()
            };
            let Value::Array(array) = eval_var(&array_access.array, env) else {
                unreachable!()
            };

            let index = eval_array_index(index, array.len());

            array[index].clone()
        }
    }
}

pub fn eval_array_index(index: i32, arr_len: usize) -> usize {
    usize::try_from(index)
        .ok()
        .filter(|&i| i < arr_len)
        .unwrap_or_else(|| panic!("index out of bounds for array length {arr_len}: {index}"))
}

pub fn eval_binary<'a>(
    binary_expression: &BinaryExpression,
    env: Rc<Environment<'a>>,
) -> Value<'a> {
    let op1 = eval_expression(&binary_expression.left, env.clone());
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

pub fn eval_unary<'a>(unary: &UnaryExpression, env: Rc<Environment<'a>>) -> Value<'a> {
    let op = eval_expression(&unary.operand, env);

    match unary.operator {
        unary_expression::UnaryOperator::Minus => -op,
    }
}
