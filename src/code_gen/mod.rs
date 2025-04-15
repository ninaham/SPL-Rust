use crate::{
    absyn::absyn::{Definition, Program},
    table::{symbol_table::SymbolTable, types::Type},
};
use std::{collections::HashMap, fmt};
mod procedure_def;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Quadrupel {
    pub op: QuadrupelOp,
    pub arg1: String,
    pub arg2: String,
    pub result: String,
}

pub struct Tac<'a> {
    pub quadrupels: Vec<Quadrupel>,
    ast: &'a mut Program,
    symboltable: &'a SymbolTable,
    label_stack: Vec<i64>,
    label_num: i64,
    label_table: HashMap<String, i64>,
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
    pub fn new(ast: &'a mut Program, symboltable: &'a SymbolTable) -> Self {
        Tac {
            quadrupels: vec![],
            ast,
            symboltable,
            label_stack: vec![],
            label_num: 0,
            label_table: HashMap::new(),
            temp_var_stack: vec![],
            temp_var_count: 0,
        }
    }

    pub fn code_generation(&mut self) {
        for definition in &self.ast.definitions {
            match definition.as_ref() {
                Definition::ProcedureDefinition(proc_def) => {
                    self.eval_proc_def(proc_def);
                }
                _ => {}
            }
        }
    }
}
