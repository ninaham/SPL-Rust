use colored::Colorize;
use quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar};

use crate::{
    absyn::absyn::{Definition, Program},
    table::symbol_table::SymbolTable,
};
use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};
mod procedure_def;
pub mod quadrupel;
mod utils;

#[derive(Clone)]
pub struct Tac {
    pub quadrupels: Vec<Quadrupel>,
    pub symboltable: Rc<RefCell<SymbolTable>>,
    label_num: i64,
    pub proc_table: HashMap<String, Vec<Quadrupel>>,
    temp_var_count: usize,
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
    pub fn new(symboltable: Rc<RefCell<SymbolTable>>) -> Self {
        Self {
            quadrupels: vec![],
            symboltable,
            label_num: 0,
            proc_table: HashMap::new(),
            temp_var_count: 0,
        }
    }

    pub fn code_generation(&mut self, ast: &Program) {
        let definitions: Vec<_> = ast.definitions.iter().collect();
        for definition in definitions {
            let name: String;
            match definition.as_ref() {
                Definition::ProcedureDefinition(proc_def) => {
                    name = proc_def.name.clone();
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
}
