use std::rc::Rc;

use crate::{
    absyn::{
        absyn::{Expression, Statement},
        assign_statement::AssignStatement,
        call_statement::CallStatement,
        if_statement::IfStatement,
        while_statement::WhileStatement,
    },
    interpreter::{
        definition_evaluator::eval_local_var,
        environment::Environment,
        expression_evaluator::{eval_expression, eval_var},
        value::{Value, ValueFunction},
    },
    table::{entry::Entry, symbol_table::SymbolTable},
};

// Evaluates any statement and executes it in the given environment.
pub fn eval_statement<'a, 'b: 'a>(
    statement: &'b Statement,
    table: &SymbolTable,
    env: Rc<Environment<'b, '_>>,
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
            for s in statements {
                eval_statement(s, table, env.clone());
            }
        }
    }
}

// Executes an if statement.
pub fn eval_if_statement<'a, 'b: 'a>(
    statement: &'b IfStatement,
    table: &SymbolTable,
    env: Rc<Environment<'b, '_>>,
) {
    // Evaluate the condition of the if statement.
    let cond = eval_expression(&statement.condition, env.clone());

    // Match the condition value to determine which branch to execute.
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

// Executes an assignment statement, evaluating the value and assigning it to the variable.
pub fn eval_assign_statement<'a, 'b: 'a>(
    statement: &AssignStatement,
    env: &Rc<Environment<'b, '_>>,
) {
    // Evaluate the value of the assignment statement.
    let val = eval_expression(&statement.value, env.clone());
    // Get the variable reference to assign the value to.
    let x = eval_var(&statement.target, env);
    *x.borrow_mut() = val;
}

// Executes a while statement.
pub fn eval_while_statement<'a, 'b: 'a>(
    statement: &'b WhileStatement,
    table: &SymbolTable,
    env: &Rc<Environment<'b, '_>>,
) {
    // Evaluate the condition of the while statement. Execute the body as long as the condition is true.
    while eval_expression(&statement.condition, env.clone()) == Value::Bool(true) {
        // Execute the body of the while statement.
        eval_statement(&statement.body, table, env.clone());
    }
}

// Calls a procedure defined in the symbol table, passing the evaluated arguments.
pub fn eval_call_statement<'a, 'b: 'a>(
    statement: &'b CallStatement,
    table: &SymbolTable,
    env: &Rc<Environment<'a, '_>>,
) {
    // Look up the procedure in the symbol table.
    let Some(Value::Function(proc)) = env.get(&statement.name).map(|v| v.borrow().clone()) else {
        unimplemented!("SPL-builtin `{}()`", statement.name);
    };

    // Get arguments for the procedure call, evaluating each argument based on the procedure's parameters.
    let args = statement
        .arguments
        .iter()
        .zip(&proc.entry().parameters)
        .map(|(e, p)| {
            if p.is_reference {
                let Expression::VariableExpression(var) = e else {
                    unreachable!()
                };
                eval_var(var, env)
            } else {
                Value::new_refcell(eval_expression(e, env.clone()))
            }
        })
        .collect::<Vec<_>>();

    // Match the procedure type and execute it accordingly.
    match proc {
        ValueFunction::Spl(proc_entry, proc_body) => {
            // Get the local symbol table for the procedure.
            let local_table = match table.lookup(&statement.name).unwrap() {
                Entry::ProcedureEntry(procedure_entry) => procedure_entry.local_table,
                _ => unreachable!(),
            };

            // get the parameters for the procedure call and create a new environment with the parameters and local variables.
            let vars_param = proc_entry
                .parameters
                .iter()
                .zip(args)
                .map(|(var, arg)| (var.name.clone(), arg));
            let vars_param_names = vars_param.clone().map(|(n, _)| n).collect::<Vec<_>>();

            let vars_local = proc_entry
                .local_table
                .entries
                .keys()
                .filter(|&n| !vars_param_names.contains(n))
                .map(|var_name| eval_local_var(var_name, &local_table));

            let new_env = Rc::new(Environment::new(
                env.clone(),
                vars_param.chain(vars_local),
                &local_table,
            ));

            // Execute the procedure body in the new environment.
            for s in proc_body {
                eval_statement(s, &local_table, new_env.clone());
            }
        }
        ValueFunction::BuiltIn(_, f) => {
            // Call the built-in function with the evaluated arguments.
            f.call(&args);
        }
        ValueFunction::Tac(_, _) => unreachable!(),
    }
}
