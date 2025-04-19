use colored::Colorize;
use std::fmt;

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
