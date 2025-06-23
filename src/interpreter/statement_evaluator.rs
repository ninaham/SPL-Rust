use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    absyn::{
        absyn::{Statement, Variable},
        assign_statement::AssignStatement,
        call_statement::CallStatement,
        if_statement::IfStatement,
        while_statement::WhileStatement,
    },
    interpreter::{
        definition_evaluator::eval_local_var, environment::Environment,
        expression_evaluator::eval_array_index, expression_evaluator::eval_expression,
        value::Value, value::ValueFunction,
    },
    table::{entry::Entry, symbol_table::SymbolTable},
};

pub fn eval_statement<'a, 'b: 'a>(
    statement: &Statement,
    table: &SymbolTable,
    env: Rc<Environment<'b>>,
) {
    match statement {
        Statement::AssignStatement(assign_statement) => {
            eval_assign_statement(assign_statement, &env);
        }
        Statement::IfStatement(if_statement) => eval_if_statement(if_statement, table, env),
        Statement::WhileStatement(while_statement) => {
            eval_while_statement(while_statement, table, &env);
        }
        Statement::CallStatement(call_statement) => {
            eval_call_statement(call_statement, table, &env);
        }
        Statement::EmptyStatement => (),
        Statement::CompoundStatement(statements) => {
            statements
                .iter()
                .for_each(|s| eval_statement(s, table, env.clone()));
        }
    }
}

pub fn eval_if_statement<'a, 'b: 'a>(
    statement: &IfStatement,
    table: &SymbolTable,
    env: Rc<Environment<'b>>,
) {
    let cond = eval_expression(&statement.condition, env.clone());

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

pub fn eval_assign_statement<'a, 'b: 'a>(statement: &AssignStatement, env: &Rc<Environment<'b>>) {
    let val = eval_expression(&statement.value, env.clone());
    eval_var_mut(&statement.target, env, &|var| {
        *var = val.clone();
    });
}

pub fn eval_var_mut<'a>(
    variable: &Variable,
    env: &Rc<Environment<'a>>,
    f: &dyn Fn(&mut Value<'a>),
) {
    match variable {
        Variable::NamedVariable(var_name) => f(env
            .vars
            .borrow_mut()
            .get_mut(var_name)
            .unwrap_or_else(|| panic!("not found: {var_name}"))),
        Variable::ArrayAccess(array_access) => {
            let Value::Int(index) = eval_expression(&array_access.index, env.clone()) else {
                unreachable!()
            };
            eval_var_mut(&array_access.array, env, &move |a| {
                let Value::Array(a) = a else { unreachable!() };
                let index = eval_array_index(index, a.len());
                f(&mut a[index]);
            });
        }
    }
}

pub fn eval_while_statement<'a, 'b: 'a>(
    statement: &WhileStatement,
    table: &SymbolTable,
    env: &Rc<Environment<'b>>,
) {
    while eval_expression(&statement.condition, env.clone()) == Value::Bool(true) {
        eval_statement(&statement.body, table, env.clone());
    }
}

pub fn eval_call_statement<'a, 'b: 'a>(
    statement: &CallStatement,
    table: &SymbolTable,
    env: &Rc<Environment>,
) {
    let args = statement
        .arguments
        .iter()
        .map(|e| eval_expression(e, env.clone()))
        .collect::<Vec<_>>();

    let new_env = Rc::new(Environment {
        parent: Some(env.clone()),
        vars: RefCell::new(HashMap::new()),
    });

    let Value::Function(proc) = env.get(&statement.name).unwrap() else {
        unreachable!()
    };

    match proc {
        ValueFunction::Spl(proc) => {
            let local_table = match table.lookup(&statement.name).unwrap() {
                Entry::ProcedureEntry(procedure_entry) => procedure_entry.local_table,
                _ => unreachable!(),
            };

            for var in &proc.variables {
                eval_local_var(var, &local_table, &new_env.clone());
            }

            for (i, var) in proc.parameters.iter().enumerate() {
                new_env.insert(&var.name, args[i].clone());
            }

            for s in &proc.body {
                eval_statement(s, &local_table, new_env.clone());
            }
        }
        ValueFunction::BuiltIn(f) => f(&args),
    }
}
