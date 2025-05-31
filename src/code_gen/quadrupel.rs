use colored::Colorize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        if self == &Self::Default {
            write!(f, "               ")
        } else {
            write!(
                f,
                "{:<15}",
                format!("{self:?}").to_uppercase().bright_blue()
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
impl PartialEq for QuadrupelArg {
    fn eq(&self, other: &Self) -> bool {
        use QuadrupelArg::{Const, Empty, Var};
        match (self, other) {
            (Var(a), Var(b)) => a == b,
            (Const(a), Const(b)) => a == b,
            (Empty, Empty) => true,
            _ => false,
        }
    }
}
impl Eq for QuadrupelArg {}

impl fmt::Display for QuadrupelArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{:<15}", var.to_string().truecolor(150, 150, 150)),
            Self::Const(val) => write!(f, "{val:<15}"),
            Self::Empty => write!(f, "               "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuadrupelVar {
    Spl(String),
    Tmp(usize),
}

impl fmt::Display for QuadrupelVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spl(var) => write!(f, "{var:<15}"),
            Self::Tmp(val) => write!(f, "T:{val:<13}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuadrupelResult {
    Var(QuadrupelVar),
    Label(String),
    Empty,
}

impl PartialEq<QuadrupelArg> for QuadrupelResult {
    fn eq(&self, other: &QuadrupelArg) -> bool {
        let Self::Var(self_var) = self else {
            return false;
        };
        let QuadrupelArg::Var(other_var) = other else {
            return false;
        };

        self_var == other_var
    }
}

impl fmt::Display for QuadrupelResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(var) => write!(f, "{}", var.to_string().truecolor(200, 200, 200)),
            Self::Label(name) => write!(f, "{name}"),
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

macro_rules! quad {
    (=> $label:expr) => {quad!((=>), _, _ => $label)};

    (($($op:tt)+), $arg1:tt, $arg2:tt => $result:expr ) => {{
        $crate::code_gen::quadrupel::Quadrupel {
            op: quad!(@op $($op)+),
            arg1: quad!(@arg $arg1),
            arg2: quad!(@arg $arg2),
            result: $result,
        }
    }};


    (@@op $o:ident) => {$crate::code_gen::quadrupel::QuadrupelOp::$o};
    (@op +   ) => { quad!(@@op Add       ) };
    (@op -   ) => { quad!(@@op Sub       ) };
    (@op *   ) => { quad!(@@op Mul       ) };
    (@op /   ) => { quad!(@@op Div       ) };
    (@op ~   ) => { quad!(@@op Neg       ) };
    (@op ==  ) => { quad!(@@op Equ       ) };
    (@op !=  ) => { quad!(@@op Neq       ) };
    (@op <   ) => { quad!(@@op Lst       ) };
    (@op <=  ) => { quad!(@@op Lse       ) };
    (@op >   ) => { quad!(@@op Grt       ) };
    (@op >=  ) => { quad!(@@op Gre       ) };
    (@op :=  ) => { quad!(@@op Assign    ) };
    (@op =[] ) => { quad!(@@op ArrayLoad ) };
    (@op []= ) => { quad!(@@op ArrayStore) };
    (@op =>  ) => { quad!(@@op Goto      ) };
    (@op p   ) => { quad!(@@op Param     ) };
    (@op c   ) => { quad!(@@op Call      ) };
    (@op d   ) => { quad!(@@op Default   ) };
    (@op _   ) => { _                      };

    (@@arg $E:ident $(($a:tt))?) => {$crate::code_gen::quadrupel::QuadrupelArg::$E$(($a))?};
    (@arg ($arg:expr)  ) => {           $arg           };
    (@arg $val:literal ) => { quad!(@@arg Const($val)) };
    (@arg (=$val:tt)   ) => { quad!(@@arg Const($val)) };
    (@arg (~$val:tt)   ) => { quad!(@@arg Var($val)  ) };
    (@arg _            ) => { quad!(@@arg Empty      ) };
}
pub(crate) use quad;

macro_rules! quad_match {
    (($($op:tt)+) $(($($ops:tt)+))*, $arg1:tt, $arg2:tt => $result:pat ) => {

        $crate::code_gen::quadrupel::Quadrupel {
            op: quad!(@op $($op)+) $(| quad!(@op $($ops)+))*,
            arg1: quad_match!(@arg $arg1),
            arg2: quad_match!(@arg $arg2),
            result: $result,
        }
    };

    (@arg $arg:ident      ) => { $arg                 };
    (@arg $val:literal    ) => { quad!(@arg $val    ) };
    (@arg ($o:tt$val:pat) ) => { quad!(@arg ($o$val)) };
    (@arg _               ) => { _                    };
}
pub(crate) use quad_match;
