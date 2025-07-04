use std::{collections::LinkedList, rc::Rc};

use crate::{
    absyn::{
        absyn::{Definition, Program},
        call_statement::CallStatement,
    },
    interpreter::{
        environment::Environment,
        statement_evaluator::eval_call_statement,
        value::{Value, ValueFunction},
    },
    spl_builtins,
    table::{entry::Entry, symbol_table::SymbolTable},
};

use super::value::ValueRef;

//Interprets the given program and returns a new global environment with procedures defined in it.
pub fn eval_program<'a>(
    program: &'a Program,
    symbol_table: &'a SymbolTable,
) -> Environment<'a, 'a> {
    let procs = program
        .definitions
        .iter()
        .filter_map(|def| match def.as_ref() {
            Definition::ProcedureDefinition(proc_def) => Some(proc_def),
            Definition::TypeDefinition(_) => None,
        })
        .map(|proc_def| {
            let Some(Entry::ProcedureEntry(proc_entry)) = symbol_table.lookup(&proc_def.name)
            else {
                unreachable!();
            };
            (
                proc_def.name.to_string(),
                Value::new_refcell(Value::Function(ValueFunction::Spl(
                    proc_entry,
                    &proc_def.body,
                ))),
            )
        });

    Environment::new_global(procs, symbol_table)
}

// Starts the main procedure of the given program, initializing the environment and executing it.
pub fn start_main(program: &Program, table: &SymbolTable) {
    //Initialize the environment with the program's procedures
    let env = Rc::new(eval_program(program, table));

    let call_stmt = CallStatement {
        name: "main".to_string(),
        arguments: LinkedList::new(),
    };

    //Initialize program start for built-in time function
    spl_builtins::init_start_time();

    //Evaluate the call statement for the main procedure
    eval_call_statement(&call_stmt, table, &env);
}

// Evaluates a local variable by looking it up in the symbol table and returning its name and default value.
pub fn eval_local_var<'a>(var_name: &str, table: &SymbolTable) -> (String, ValueRef<'a>) {
    let Entry::VariableEntry(var_ent) = table.lookup(var_name).unwrap() else {
        unreachable!()
    };

    (
        var_name.to_string(),
        Value::new_refcell(var_ent.typ.default_value()),
    )
}
