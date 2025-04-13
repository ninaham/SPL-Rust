use crate::{
    absyn::absyn::{Definition, Program},
    table::symbol_table::SymbolTable,
};
use std::fmt;
mod procedure_def;
mod type_def;

#[derive(Debug)]
pub enum QuatrupelOp {
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
    Param,
    Call, // call p, n
}

impl fmt::Display for QuatrupelOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuatrupelOp::Label(label) => write!(f, "{}:", label),
            other => write!(f, "{}", format!("{:?}", other).to_lowercase()),
        }
    }
}

#[derive(Debug)]
pub struct Quatrupels {
    pub op: QuatrupelOp,
    pub arg1: String,
    pub arg2: String,
    pub result: String,
}

impl fmt::Display for Quatrupels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.op {
            QuatrupelOp::Label(_) => {
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

pub fn code_generation(ast: &mut Program, symboltable: &SymbolTable) -> Vec<Quatrupels> {
    let mut quad_code: Vec<Quatrupels> = vec![];
    while let Some(definition) = ast.definitions.pop_front() {
        match *definition {
            Definition::ProcedureDefinition(proc_def) => {}
            Definition::TypeDefinition(type_def) => {}
        }
    }
    quad_code
}
