use std::{collections::HashMap, rc::Weak, sync::Mutex};

use crate::semant::SemanticError;

use super::entry::Entry;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub entries: HashMap<String, Entry>,
    pub upper_level: Option<Weak<Mutex<SymbolTable>>>,
}

impl SymbolTable {
    pub fn enter(&mut self, name: String, entry: Entry) -> Result<(), SemanticError> {
        if self.entries.contains_key(&name) {
            return Err(SemanticError {
                _msg: format!("Symbol {} already defined", name),
            });
        }
        self.entries.insert(name, entry);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<Entry> {
        if let Some(entry) = self.entries.get(name) {
            return Some(entry.clone());
        }
        if let Some(upper_level) = self.upper_level.clone() {
            let u_l = upper_level.upgrade().unwrap();
            let u_l = u_l.lock().unwrap();
            return u_l.lookup(name);
        }
        None
    }
}
