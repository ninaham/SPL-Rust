use std::collections::HashMap;

use crate::{interpreter::value::Value, table::symbol_table::SymbolTable};

pub struct Environment {
    parent: Box<Environment>,
    vars: HashMap<String, Value>,
}

impl Environment {
    pub fn new(table: &SymbolTable) -> Self {
        todo!()
    }

    pub fn get(&self, key: &String) -> Option<Value> {
        todo!()
    }

    pub fn insert(&mut self, key: &String, value: Value) {}
}
