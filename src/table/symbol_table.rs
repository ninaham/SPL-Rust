use std::{collections::HashMap, fmt::Error};

use super::entry::Entry;

#[derive(Debug)]
pub struct SymbolTable {
    pub entries: HashMap<String, Entry>,
    pub upper_level: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn enter(&mut self, name: String, entry: Entry) -> Result<(), Error> {
        // Check if the name already exists in the current symbol table, define SPLError if necessary
        self.entries.insert(name, entry);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Entry> {
        if let Some(entry) = self.entries.get(name) {
            return Some(entry);
        }
        if let Some(upper_level) = &self.upper_level {
            return upper_level.lookup(name);
        }
        None
    }
}
