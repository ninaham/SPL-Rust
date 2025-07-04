use std::rc::Rc;

use crate::{
    absyn::{
        absyn::{Expression, Variable},
        binary_expression::{self, BinaryExpression},
        unary_expression::{self, UnaryExpression},
    },
    interpreter::{environment::Environment, value::Value},
};

use super::value::ValueRef;

// Evaluates any expression and returns its value.
pub fn eval_expression<'a>(expression: &Expression, env: Rc<Environment<'a, '_>>) -> Value<'a> {
    match expression {
        Expression::BinaryExpression(binary_expression) => eval_binary(binary_expression, env),
        Expression::UnaryExpression(unary_expression) => eval_unary(unary_expression, env),
        Expression::IntLiteral(i) => Value::Int(*i),
        Expression::VariableExpression(variable) => eval_var(variable, &env).borrow().clone(),
    }
}

// Evaluates a variable expression and returns its value.
pub fn eval_var<'a>(variable: &Variable, env: &Rc<Environment<'a, '_>>) -> ValueRef<'a> {
    match variable {
        Variable::NamedVariable(v) => env.get(v).unwrap(),
        Variable::ArrayAccess(array_access) => {
            // Evaluate the index. Make sure it is an integer.
            let Value::Int(index) = eval_expression(&array_access.index, env.clone()) else {
                unreachable!()
            };
            // Evaluate the array and ensure it is an array type.
            let array = eval_var(&array_access.array, env);
            let Value::Array(ref array) = *array.borrow() else {
                unreachable!()
            };

            // Ensure the index is within bounds.
            let index = eval_array_index(index, array.len());

            // Return the value at the specified index.
            array[index].clone()
        }
    }
}

// Evaluates an array index, ensuring it is within bounds.
pub fn eval_array_index(index: i32, arr_len: usize) -> usize {
    usize::try_from(index)
        .ok()
        .filter(|&i| i < arr_len)
        .unwrap_or_else(|| panic!("index out of bounds for array length {arr_len}: {index}"))
}

// Evaluates a binary expression and returns its value.
pub fn eval_binary<'a>(
    binary_expression: &BinaryExpression,
    env: Rc<Environment<'a, '_>>,
) -> Value<'a> {
    // Evaluate the left and right operands of the binary expression.
    let op1 = eval_expression(&binary_expression.left, env.clone());
    let op2 = eval_expression(&binary_expression.right, env);

    // Calculate the result based on the operator.
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

// Evaluates a unary expression and returns its value.
pub fn eval_unary<'a>(unary: &UnaryExpression, env: Rc<Environment<'a, '_>>) -> Value<'a> {
    let op = eval_expression(&unary.operand, env);

    match unary.operator {
        unary_expression::UnaryOperator::Minus => -op,
    }
}
