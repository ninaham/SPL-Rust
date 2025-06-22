use crate::{
    absyn::{
        absyn::{Statement, Variable},
        assign_statement::AssignStatement,
        call_statement::CallStatement,
        if_statement::IfStatement,
        while_statement::WhileStatement,
    },
    interpreter::{environment::Environment, expression_evaluator::eval_expression, value::Value},
    table::{entry::Entry, symbol_table::SymbolTable},
};

pub fn eval_statement(statement: &Statement, table: &SymbolTable, env: &mut Environment) {
    match statement {
        Statement::AssignStatement(assign_statement) => {
            eval_assign_statement(assign_statement, table, env);
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

pub fn eval_if_statement(statement: &IfStatement, table: &SymbolTable, env: &mut Environment) {
    let cond = eval_expression(&statement.condition, env);

    match cond {
        Value::Int(_) | Value::Array(_) => unreachable!(),
        Value::Bool(b) => {
            if b {
                eval_statement(&statement.then_branch, table, env);
            } else if let Some(s) = statement.else_branch.clone() {
                eval_statement(&s, table, env);
            }
        }
    }
}

pub fn eval_assign_statement(
    statement: &AssignStatement,
    table: &SymbolTable,
    env: &mut Environment,
) {
    let val = eval_expression(&statement.value, env);

    match statement.target.clone() {
        Variable::NamedVariable(k) => {
            env.vars.insert(k, val);
        }
        Variable::ArrayAccess(array_access) => todo!(),
    }
}

pub fn eval_while_statement(
    statement: &WhileStatement,
    table: &SymbolTable,
    env: &mut Environment,
) {
    while let Value::Bool(b) = eval_expression(&statement.condition, env) {
        if !b {
            break;
        }
        eval_statement(&statement.body, table, env);
    }
}

pub fn eval_call_statement(statement: &CallStatement, table: &SymbolTable, env: &mut Environment) {
    let args = statement
        .arguments
        .iter()
        .map(|e| eval_expression(e, env))
        .collect::<Vec<_>>();

    let proc = env.procs.get(&statement.name).unwrap().clone();

    proc.iter().for_each(|s| eval_statement(s, table, env));
}
