use std::{
    cell::RefCell,
    collections::{HashMap, LinkedList},
    rc::Rc,
};

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

pub fn eval_program(program: &Program) -> Environment {
    let env = Environment {
        parent: None,
        vars: RefCell::new(HashMap::new()),
    };

    get_builtins()
        .iter()
        .for_each(|b| env.insert(b.0, b.1.clone()));

    for def in &program.definitions {
        match def.as_ref() {
            Definition::ProcedureDefinition(procedure_definition) => {
                env.insert(
                    &procedure_definition.name,
                    Value::Function(ValueFunction::Spl(procedure_definition)),
                );
            }
            Definition::TypeDefinition(_) => {}
        }
    }

    env
}

pub fn start_main(program: &Program, table: &SymbolTable) {
    let env = Rc::new(eval_program(program));
    let call_stmt = CallStatement {
        name: "main".to_string(),
        arguments: LinkedList::new(),
    };
    eval_call_statement(&call_stmt, table, &env);
}

pub fn eval_local_var(var: &VariableDefinition, table: &SymbolTable, env: &Environment) {
    let Entry::VariableEntry(var_ent) = table.lookup(&var.name).unwrap() else {
        unreachable!()
    };
    env.insert(&var.name, var_ent.typ.default_value());
}

fn get_builtins<'a>() -> [(&'static str, Value<'a>); 2] {
    [
        (
            "printi",
            Value::new_builtin_proc(&[("i", false)], |v: &[Value]| {
                print!(
                    "{}",
                    match v[0] {
                        Value::Int(i) => i,
                        _ => unreachable!(),
                    }
                );
            }),
        ),
        (
            "printc",
            Value::new_builtin_proc(&[("c", false)], |v: &[Value]| {
                let c = match v[0] {
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
