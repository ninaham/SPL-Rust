use colored::Colorize;
use quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar};

use crate::{
    absyn::absyn::{Definition, Program},
    table::{entry::Entry, symbol_table::SymbolTable},
};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    fmt,
    rc::Rc,
};
mod procedure_def;
pub mod quadrupel;
mod utils;

/// Struct representing the Three-Address Code (TAC) generator and storage.
pub struct Tac {
    /// Vector holding the currently generated quadruples.
    quadrupels: Vec<Quadrupel>,
    /// Counter for generating unique labels.
    label_num: i64,
    /// Mapping from procedure names to their respective quadruple lists.
    pub proc_table: HashMap<String, Vec<Quadrupel>>,
    /// Counter to generate unique temporary variable names.
    temp_var_count: usize,
    /// Reference-counted and mutable global symbol table.
    global_table: Rc<RefCell<SymbolTable>>,
    /// Name of the procedure currently being processed (if any).
    current_proc: Option<String>,
}

impl fmt::Display for Tac {
    /// Implements pretty printing for the TAC.
    /// Prints all procedures and their quadruples with separators.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for proc in &self.proc_table {
            for quad in proc.1 {
                writeln!(f, "{quad}")?;
            }
            // Print a separator line with gray color between procedures
            writeln!(f, "{:-<58}", "".truecolor(100, 100, 100))?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Tac {
    /// Creates a new TAC generator instance with the given global symbol table.
    pub fn new(global_table: Rc<RefCell<SymbolTable>>) -> Self {
        Self {
            quadrupels: vec![],
            label_num: 0,
            proc_table: HashMap::new(),
            temp_var_count: 0,
            global_table,
            current_proc: None,
        }
    }

    /// Generates three-address code from the given abstract syntax tree (AST) program.
    /// Iterates over all definitions and processes procedure definitions to generate code.
    pub fn code_generation(&mut self, ast: &Program) {
        // Collect all definitions from the AST into a vector for iteration
        let definitions: Vec<_> = ast.definitions.iter().collect();
        for definition in definitions {
            let name: String;
            match definition.as_ref() {
                // If it's a procedure definition, generate code for it
                Definition::ProcedureDefinition(proc_def) => {
                    name = proc_def.name.clone();
                    self.current_proc = Some(name.clone());
                    self.eval_proc_def(proc_def);

                    // Clone the generated quadruples and store in the procedure table
                    let quad: Vec<_> = self.quadrupels.clone();
                    self.proc_table.insert(name, quad);

                    // Clean up for next procedure generation
                    self.quadrupels.clear();
                    self.temp_var_count = 0;
                }
                // Type definitions do not generate code, so ignore
                Definition::TypeDefinition(_) => {}
            }
        }
    }

    /// Accesses the local symbol table of the currently processed procedure.
    /// Uses RefMut to allow mutable borrowing with proper lifetime.
    fn local_table(&self) -> RefMut<'_, SymbolTable> {
        RefMut::map(self.global_table.borrow_mut(), |b| {
            match b
                .entries
                .get_mut(self.current_proc.as_ref().unwrap())
                .unwrap()
            {
                Entry::ProcedureEntry(proc_entry) => &mut proc_entry.local_table,
                _ => unreachable!(),
            }
        })
    }
}
