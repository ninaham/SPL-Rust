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

pub struct Tac {
    quadrupels: Vec<Quadrupel>,
    label_num: i64,
    pub proc_table: HashMap<String, Vec<Quadrupel>>,
    temp_var_count: usize,
    global_table: Rc<RefCell<SymbolTable>>,
    current_proc: Option<String>,
}

impl fmt::Display for Tac {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for proc in &self.proc_table {
            for quad in proc.1 {
                writeln!(f, "{quad}")?;
            }
            writeln!(f, "{:-<58}", "".truecolor(100, 100, 100))?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Tac {
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

    pub fn code_generation(&mut self, ast: &Program) {
        let definitions: Vec<_> = ast.definitions.iter().collect();
        for definition in definitions {
            let name: String;
            match definition.as_ref() {
                Definition::ProcedureDefinition(proc_def) => {
                    name = proc_def.name.clone();
                    self.current_proc = Some(name.clone());
                    self.eval_proc_def(proc_def);

                    let quad: Vec<_> = self.quadrupels.clone();
                    self.proc_table.insert(name, quad);
                    //clean up for next iteration
                    self.quadrupels.clear();
                    self.temp_var_count = 0;
                }
                Definition::TypeDefinition(_) => {}
            }
        }
    }

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
