use core::str;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::value::Value,
    spl_builtins::PROCEDURES,
    table::{entry::Entry, symbol_table::SymbolTable},
};

use super::value::ValueRef;

// Represents an environment for variable storage
#[derive(Clone, Debug)]
pub struct Environment<'a, 'b> {
    pub parent: Option<Rc<Environment<'a, 'b>>>,
    pub vars: RefCell<HashMap<String, ValueRef<'a>>>,
    symbol_table: &'b SymbolTable,
}

impl<'a, 'b> Environment<'a, 'b> {
    // Creates a new environment with a parent and variables
    pub fn new(
        parent: Rc<Self>,
        vars_iter: impl Iterator<Item = (String, ValueRef<'a>)>,
        symbol_table: &'b SymbolTable,
    ) -> Self {
        Self {
            parent: Some(parent),
            vars: RefCell::new(vars_iter.collect()),
            symbol_table,
        }
    }

    // Creates a new global environment with procedures
    pub fn new_global(
        procedures: impl Iterator<Item = (String, ValueRef<'a>)>,
        symbol_table: &'b SymbolTable,
    ) -> Self {
        Self {
            parent: None,
            vars: RefCell::new(get_builtins().chain(procedures).collect()),
            symbol_table,
        }
    }

    // Tries to recursively get a variable by key, searching through parent environments if necessary
    fn recursive_get(&self, key: &str) -> Option<ValueRef<'a>> {
        self.vars.borrow().get(key).map_or_else(
            || {
                self.parent
                    .clone()
                    .map_or_else(|| None, |p| p.recursive_get(key))
            },
            |v| Some(v.clone()),
        )
    }

    // Calls recursive_get to find a variable, returning its value if found
    pub fn get(&self, key: &str) -> Option<ValueRef<'a>> {
        if let Some(v) = self.recursive_get(key) {
            return Some(v.clone());
        }

        if let Some(Entry::VariableEntry(ve)) = self.symbol_table.lookup(key) {
            let typ = ve.typ;
            let v = self
                .vars
                .borrow_mut()
                .entry(key.to_string())
                .insert_entry(Value::new_refcell(typ.default_value()))
                .get()
                .clone();

            return Some(v);
        }

        None
    }

    // Gets a mutable reference to a variable, returning its value and whether it is a reference
    pub fn get_mut(&self, key: &str) -> Option<(ValueRef<'a>, bool)> {
        if let Some(v) = self.get(key) {
            let is_ref = if let Some(Entry::VariableEntry(ve)) = self.symbol_table.lookup(key) {
                ve.is_reference
            } else {
                false
            };
            return Some((v, is_ref));
        }

        None
    }
}

// Returns an iterator over built-in procedures
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
