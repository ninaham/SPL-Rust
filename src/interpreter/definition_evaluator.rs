use std::{collections::LinkedList, rc::Rc};

use crate::{
    absyn::{
        absyn::{Definition, Program},
        call_statement::CallStatement,
        variable_definition::VariableDefinition,
    },
    interpreter::{
        environment::Environment,
        statement_evaluator::eval_call_statement,
        value::{Value, ValueFunction},
    },
    spl_builtins::{self, PROCEDURES},
    table::{entry::Entry, symbol_table::SymbolTable},
};

use super::value::ValueRef;

pub fn eval_program(program: &'_ Program) -> Environment<'_> {
    let procs_builtin = get_builtins();

    let procs_spl = program
        .definitions
        .iter()
        .filter_map(|def| match def.as_ref() {
            Definition::ProcedureDefinition(proc_def) => Some(proc_def),
            Definition::TypeDefinition(_) => None,
        })
        .map(|proc_def| {
            (
                proc_def.name.to_string(),
                Value::new_refcell(Value::Function(ValueFunction::Spl(proc_def))),
            )
        });

    Environment::new(None, procs_builtin.chain(procs_spl))
}

pub fn start_main(program: &Program, table: &SymbolTable) {
    let env = Rc::new(eval_program(program));
    let call_stmt = CallStatement {
        name: "main".to_string(),
        arguments: LinkedList::new(),
    };

    spl_builtins::init_start_time();

    eval_call_statement(&call_stmt, table, &env);
}

pub fn eval_local_var<'a>(var: &VariableDefinition, table: &SymbolTable) -> (String, ValueRef<'a>) {
    let Entry::VariableEntry(var_ent) = table.lookup(&var.name).unwrap() else {
        unreachable!()
    };

    (
        var.name.to_string(),
        Value::new_refcell(var_ent.typ.default_value()),
    )
}

pub fn get_builtins<'a>() -> impl Iterator<Item = (String, ValueRef<'a>)> {
    PROCEDURES.iter().filter_map(|&(name, params, body)| {
        body.map(|body| {
            let params = params.iter().map(|p| (p.name.to_string(), p.is_reference));
            (
                name.to_string(),
                Value::new_refcell(Value::new_builtin_proc(params, body)),
            )
        })
    })
}
