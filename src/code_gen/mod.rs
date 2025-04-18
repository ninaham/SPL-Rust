#![expect(dead_code)]
use colored::Colorize;

use crate::{
    absyn::absyn::{Definition, Program},
    table::symbol_table::SymbolTable,
};
use std::{collections::HashMap, fmt};
mod procedure_def;
mod utils;

#[derive(Debug, Clone, Copy, PartialEq)]
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
    Param,
    Call, // call p, n
    Default,
}

impl fmt::Display for QuadrupelOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self == &QuadrupelOp::Default {
            write!(f, "        ")
        } else {
            write!(
                f,
                "{:<8}",
                format!("{:?}", self).to_uppercase().bright_blue()
            )
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuadrupelArg {
    Var(QuadrupelVar),
    Const(i32),
    Empty,
}

impl fmt::Display for QuadrupelArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{:<8}", var.to_string().truecolor(150, 150, 150)),
            Self::Const(val) => write!(f, "{:<8}", val),
            Self::Empty => write!(f, "        "),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuadrupelVar {
    Spl(String),
    Tmp(usize),
}

impl fmt::Display for QuadrupelVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spl(var) => write!(f, "{:<8}", var),
            Self::Tmp(val) => write!(f, "T{:<7}", val), // TODO: make temp vars unique
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuadrupelResult {
    Var(QuadrupelVar),
    Label(String),
    Empty,
}

impl fmt::Display for QuadrupelResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{}", var.to_string().truecolor(200, 200, 200)),
            Self::Label(name) => write!(f, "{}", name),
            Self::Empty => Ok(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quadrupel {
    pub op: QuadrupelOp,
    pub arg1: QuadrupelArg,
    pub arg2: QuadrupelArg,
    pub result: QuadrupelResult,
}

#[derive(Clone)]
pub struct Tac<'a> {
    pub quadrupels: Vec<Quadrupel>,
    symboltable: &'a SymbolTable,
    label_stack: Vec<i64>,
    label_num: i64,
    pub proc_table: HashMap<String, Vec<Quadrupel>>,
    temp_var_stack: Vec<i64>,
    temp_var_count: i64,
}

impl fmt::Display for Quadrupel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.op == QuadrupelOp::Default {
            return write!(f, "{}", format!("{}:", self.result).magenta());
        }
        let pipe = "|".to_string().truecolor(100, 100, 100);
        write!(
            f,
            "\t{}{pipe}{}{pipe}{}{pipe}{}",
            self.op, self.arg1, self.arg2, self.result
        )
    }
}

impl fmt::Display for Tac<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for proc in self.proc_table.iter() {
            for quad in proc.1 {
                writeln!(f, "{}", quad)?;
            }
        }
        Ok(())
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

    pub fn code_generation(&mut self, ast: &'a Program) {
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
                Definition::TypeDefinition(_) => {}
            }
        }
    }
}
