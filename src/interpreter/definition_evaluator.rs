#![expect(dead_code)]

use std::{cell::RefCell, collections::HashMap};

use crate::{
    absyn::{
        absyn::{Definition, Program},
        variable_definition::VariableDefinition,
    },
    interpreter::{environment::Environment, statement_evaluator::eval_statement, value::Value},
    table::{entry::Entry, symbol_table::SymbolTable},
};

pub fn eval_program(program: &Program) -> Environment {
    let env = Environment {
        parent: None,
        vars: RefCell::new(HashMap::new()),
    };

    for def in &program.definitions {
        match def.as_ref() {
            Definition::ProcedureDefinition(procedure_definition) => {
                env.insert(
                    &procedure_definition.name,
                    Value::Function(procedure_definition),
                );
            }
            Definition::TypeDefinition(_) => {}
        }
    }

    env
}

pub fn start_main(program: &Program, table: &SymbolTable) {
    let env = eval_program(program);
    match env.get("main").unwrap() {
        Value::Function(procedure_definition) => {
            let local_env = Environment {
                parent: Some(&env),
                vars: RefCell::new(HashMap::new()),
            };
            for s in &procedure_definition.body {
                eval_statement(s, table, &local_env);
            }
        }
        _ => unreachable!(),
    }
}

pub fn eval_local_var(var: &VariableDefinition, table: &SymbolTable, env: &Environment) {
    let Entry::VariableEntry(var_ent) = table.lookup(&var.name).unwrap() else {
        unreachable!()
    };
    env.insert(&var.name, var_ent.typ.default_value());
}
