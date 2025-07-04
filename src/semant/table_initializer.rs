use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::spl_builtins::{NAMED_TYPES, PROCEDURES};
use crate::table::{
    entry::{Entry, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::Type,
};

/// Initializes the global symbol table with built-in types and procedures.
///
/// This function inserts predefined named types and procedure declarations
/// into the provided symbol table reference.
pub fn init_symbol_table(s_t: &Rc<RefCell<SymbolTable>>) {
    // Borrow the symbol table mutably to allow modifications
    let mut symbol_table = s_t.borrow_mut();

    // Insert all built-in types (from NAMED_TYPES) into the symbol table
    for t in NAMED_TYPES {
        // Each type is inserted as a TypeEntry; for simplicity, all map to Type::INT here
        symbol_table
            .enter(
                t.to_string(),                                  // Convert type name to a String
                Entry::TypeEntry(TypeEntry { typ: Type::INT }), // Insert a type entry with INT type
            )
            .unwrap(); // Unwrap to panic on error (should not fail here)
    }

    // Insert all built-in procedures (from PROCEDURES) into the symbol table
    for (name, params, _) in PROCEDURES {
        // Each procedure is inserted as a ProcedureEntry with:
        // - An empty local symbol table (but with a reference to the global one)
        // - A list of parameter names/types (copied via to_vec)
        symbol_table
            .enter(
                name.to_string(), // Procedure name as string
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),               // Empty table for now
                        upper_level: Some(Rc::downgrade(s_t)), // Link to the enclosing symbol table
                    },
                    parameters: params.to_vec(), // Copy parameter list
                }),
            )
            .unwrap(); // Again, unwrap to handle only expected successful cases
    }
}
