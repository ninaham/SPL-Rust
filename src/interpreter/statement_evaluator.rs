use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    absyn::{
        absyn::{Expression, Statement, Variable},
        assign_statement::AssignStatement,
        call_statement::CallStatement,
        if_statement::IfStatement,
        while_statement::WhileStatement,
    },
    interpreter::{
        definition_evaluator::eval_local_var,
        environment::Environment,
        expression_evaluator::{eval_array_index, eval_expression},
        value::{Value, ValueFunction},
    },
    table::{entry::Entry, symbol_table::SymbolTable},
};

use super::value::ValueRef;

pub fn eval_statement<'a, 'b: 'a>(
    statement: &'b Statement,
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
    statement: &'b IfStatement,
    table: &SymbolTable,
    env: Rc<Environment<'b>>,
) {
    let cond = eval_expression(&statement.condition, env.clone());

    match cond {
        Value::Bool(b) => {
            if b {
                eval_statement(&statement.then_branch, table, env);
            } else if let Some(ref s) = statement.else_branch {
                eval_statement(s, table, env);
            }
        }
        _ => unreachable!(),
    }
}

pub fn eval_assign_statement<'a, 'b: 'a>(statement: &AssignStatement, env: &Rc<Environment<'b>>) {
    let val = eval_expression(&statement.value, env.clone());
    eval_var_mut(&statement.target, env, &|var| {
        var.assign(val.clone());
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
    statement: &'b WhileStatement,
    table: &SymbolTable,
    env: &Rc<Environment<'b>>,
) {
    while eval_expression(&statement.condition, env.clone()) == Value::Bool(true) {
        eval_statement(&statement.body, table, env.clone());
    }
}

pub fn eval_call_statement<'a, 'b: 'a>(
    statement: &'b CallStatement,
    table: &SymbolTable,
    env: &Rc<Environment<'a>>,
) {
    let Value::Function(proc) = env.get(&statement.name).unwrap() else {
        unreachable!()
    };

    let args = statement
        .arguments
        .iter()
        .zip(proc.parameters())
        .map(|(e, p)| {
            if p.is_reference {
                let Expression::VariableExpression(var) = e else {
                    unreachable!()
                };
                Value::Ref(ValueRef {
                    var: var.as_ref(),
                    env: env.clone(),
                })
            } else {
                eval_expression(e, env.clone())
            }
        })
        .collect::<Vec<_>>();

    let new_env = Rc::new(Environment {
        parent: Some(env.clone()),
        vars: RefCell::new(HashMap::new()),
    });

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
        ValueFunction::BuiltIn(f) => f.call(&args),
    }
}
