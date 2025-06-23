use std::{cell::RefCell, collections::HashMap};

use crate::{
    absyn::{
        absyn::{Expression, Statement, Variable},
        assign_statement::AssignStatement,
        call_statement::CallStatement,
        if_statement::IfStatement,
        while_statement::WhileStatement,
    },
    interpreter::{
        definition_evaluator::eval_local_var, environment::Environment,
        expression_evaluator::eval_expression, value::Value,
    },
    table::symbol_table::SymbolTable,
};

pub fn eval_statement<'a>(statement: &Statement, table: &SymbolTable, env: &'a Environment<'a>) {
    match statement {
        Statement::AssignStatement(assign_statement) => {
            eval_assign_statement(assign_statement, env);
        }
        Statement::IfStatement(if_statement) => eval_if_statement(if_statement, table, env),
        Statement::WhileStatement(while_statement) => {
            eval_while_statement(while_statement, table, env);
        }
        Statement::CallStatement(call_statement) => eval_call_statement(call_statement, table, env),
        Statement::EmptyStatement => (),
        Statement::CompoundStatement(statements) => {
            statements
                .iter()
                .for_each(|s| eval_statement(s, table, env));
        }
    }
}

pub fn eval_if_statement<'a>(
    statement: &IfStatement,
    table: &SymbolTable,
    env: &'a Environment<'a>,
) {
    let cond = eval_expression(&statement.condition, env);

    match cond {
        Value::Bool(b) => {
            if b {
                eval_statement(&statement.then_branch, table, env);
            } else if let Some(s) = statement.else_branch.clone() {
                eval_statement(&s, table, env);
            }
        }
        _ => unreachable!(),
    }
}

pub fn eval_assign_statement<'a>(statement: &AssignStatement, env: &'a Environment<'a>) {
    let val = eval_expression(&statement.value, env);

    match statement.target.clone() {
        Variable::NamedVariable(k) => {
            env.insert(&k, val);
        }
        Variable::ArrayAccess(array_access) => {
            let Value::Array(mut target) = eval_expression(
                &Expression::VariableExpression(Box::new(array_access.array)),
                env,
            ) else {
                unreachable!()
            };

            let Value::Int(index) = eval_expression(&array_access.index, env) else {
                unreachable!()
            };

            target[index as usize] = val;

            todo!()
        }
    }
}

pub fn eval_while_statement<'a>(
    statement: &WhileStatement,
    table: &SymbolTable,
    env: &'a Environment<'a>,
) {
    while let Value::Bool(b) = eval_expression(&statement.condition, env) {
        if !b {
            break;
        }
        eval_statement(&statement.body, table, env);
    }
}

pub fn eval_call_statement<'a>(
    statement: &CallStatement,
    table: &SymbolTable,
    env: &'a Environment<'a>,
) {
    let args = statement
        .arguments
        .iter()
        .map(|e| eval_expression(e, env))
        .collect::<Vec<_>>();

    let new_env = Environment {
        parent: Some(env),
        vars: RefCell::new(HashMap::new()),
    };

    let Value::Function(proc) = env.get(&statement.name).unwrap() else {
        unreachable!()
    };

    for var in &proc.variales {
        eval_local_var(var, table, &new_env);
    }

    for (i, var) in proc.parameters.iter().enumerate() {
        new_env.insert(&var.name, args[i].clone());
    }

    proc.body.iter().for_each(|s| eval_statement(s, table, env));
}
