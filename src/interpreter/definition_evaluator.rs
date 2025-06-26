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
    table::{entry::Entry, symbol_table::SymbolTable},
};

use super::value::ValueRef;

pub fn eval_program(program: &'_ Program) -> Environment<'_> {
    let procs_builtin = get_builtins()
        .into_iter()
        .map(|b| (b.0.to_string(), Value::new_refcell(b.1.clone())));

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

fn get_builtins<'a>() -> [(&'static str, Value<'a>); 2] {
    [
        (
            "printi",
            Value::new_builtin_proc(&[("i", false)], |v: &[ValueRef]| {
                print!(
                    "{}",
                    match *v[0].borrow() {
                        Value::Int(i) => i,
                        _ => unreachable!(),
                    }
                );
            }),
        ),
        (
            "printc",
            Value::new_builtin_proc(&[("c", false)], |v: &[ValueRef]| {
                let c = match *v[0].borrow() {
                    Value::Int(i) => u8::try_from(i).unwrap_or_else(|_| {
                        panic!("Argument to printc() should be a valid ASCII value: {i}")
                    }) as char,
                    _ => unreachable!(),
                };
                print!("{c}");
            }),
        ),
    ]
}
