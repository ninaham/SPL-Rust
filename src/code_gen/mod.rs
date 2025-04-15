#![expect(dead_code)]
use crate::{
    absyn::absyn::{Definition, Program},
    table::{symbol_table::SymbolTable, types::Type},
};
use std::{collections::HashMap, fmt};
mod procedure_def;

#[derive(Debug, Clone)]
pub enum QuadrupelOp {
    Add,
    Sub,
    Mul,
    Div,
    Neg,
    Equ,
    Neq,
    Lst,
    Lse,
    Grt,
    Gre,
    Assign,     // v := w
    ArrayLoad,  // x = y[i]   =[]
    ArrayStore, // x[i] = y   []=
    Goto,       // let the fun begin
    IfGoto,     // if x relop y goto L
    Label(String),
    Param(Type),
    Call, // call p, n
    Default,
}

impl fmt::Display for QuadrupelOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuadrupelOp::Label(label) => write!(f, "{}:", label),
            other => write!(f, "{}", format!("{:?}", other).to_uppercase()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quadrupel {
    pub op: QuadrupelOp,
    pub arg1: String,
    pub arg2: String,
    pub result: String,
}

pub struct Tac<'a> {
    pub quadrupels: Vec<Quadrupel>,
    symboltable: &'a SymbolTable,
    label_stack: Vec<i64>,
    label_num: i64,
    proc_table: HashMap<String, Vec<Quadrupel>>,
    temp_var_stack: Vec<i64>,
    temp_var_count: i64,
}

impl fmt::Display for Quadrupel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.op {
            QuadrupelOp::Label(_) => {
                write!(f, "{}", self.op)
            }
            _ => {
                write!(
                    f,
                    "{:<8} {:<8} {:<8} {:<8}",
                    self.op, self.arg1, self.arg2, self.result
                )
            }
        }
    }
}

impl<'a> Tac<'a> {
    pub fn new(symboltable: &'a SymbolTable) -> Self {
        Tac {
            quadrupels: vec![],
            symboltable,
            label_stack: vec![],
            label_num: 0,
            proc_table: HashMap::new(),
            temp_var_stack: vec![],
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
                    self.label_num = 0;
                    self.label_stack.clear();
                    self.temp_var_stack.clear();
                    self.temp_var_count = 0;
                }
                _ => {}
            }
        }
    }
}
