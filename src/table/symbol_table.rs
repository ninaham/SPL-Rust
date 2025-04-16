use std::collections::HashMap;

use crate::semant::SemanticError;

use super::entry::Entry;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub entries: HashMap<String, Entry>,
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

    pub fn lookup<'a>(
        &'a self,
        name: &str,
        upper_level: Option<&'a SymbolTable>,
    ) -> Option<&'a Entry> {
        if let Some(entry) = self.entries.get(name) {
            return Some(entry);
        }
        if let Some(upper_level) = upper_level {
            return upper_level.lookup(name, None);
        }
        None
    }
}
