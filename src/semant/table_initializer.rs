use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::spl_builtins::{NAMED_TYPES, PROCEDURES};
use crate::table::{
    entry::{Entry, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::Type,
};

pub fn init_symbol_table(s_t: &Rc<RefCell<SymbolTable>>) {
    let mut symbol_table = s_t.borrow_mut();

    for t in NAMED_TYPES {
        symbol_table
            .enter(
                t.to_string(),
                Entry::TypeEntry(TypeEntry { typ: Type::INT }),
            )
            .unwrap();
    }

    for (name, params, _) in PROCEDURES {
        symbol_table
            .enter(
                name.to_string(),
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),
                        upper_level: Some(Rc::downgrade(s_t)),
                    },
                    parameters: params.to_vec(),
                }),
            )
            .unwrap();
    }
}
