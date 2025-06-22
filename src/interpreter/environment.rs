use std::collections::{HashMap, LinkedList};

use crate::{
    absyn::absyn::{Statement, Variable},
    interpreter::value::Value,
    table::{self, symbol_table::SymbolTable},
};

pub struct Environment {
    pub statements: Vec<Statement>,
    pub vars: HashMap<String, Value>,
    pub procs: HashMap<String, LinkedList<Statement>>,
}

impl Environment {
    pub fn new(table: &SymbolTable) -> Self {
        todo!()
    }
}
