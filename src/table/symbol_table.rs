use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::sync::Mutex;

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
                msg: format!("Symbol {name} already defined"),
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

    pub fn upper_level(&self) -> Rc<Mutex<Self>> {
        self.upper_level.as_ref().unwrap().upgrade().unwrap()
    }
}
