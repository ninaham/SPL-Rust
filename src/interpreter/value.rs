use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::LinkedList,
    fmt::Debug,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

use crate::{
    absyn::absyn::Statement,
    base_blocks::BlockGraph,
    table::{
        entry::{Parameter, ProcedureEntry},
        symbol_table::SymbolTable,
        types::Type,
    },
};

pub type ValueRef<'a> = Rc<RefCell<Value<'a>>>;

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Int(i32),
    Bool(bool),
    Array(Vec<ValueRef<'a>>),
    Function(ValueFunction<'a>),
}

#[derive(Clone, Debug)]
pub enum ValueFunction<'a> {
    #[expect(clippy::linkedlist)]
    Spl(ProcedureEntry, &'a LinkedList<Statement>),
    Tac(ProcedureEntry, &'a BlockGraph),
    BuiltIn(ProcedureEntry, BuiltInProc),
}
impl ValueFunction<'_> {
    pub fn parameters(&self) -> Box<dyn Iterator<Item = &Parameter> + '_> {
        match self {
            ValueFunction::Tac(proc, _)
            | ValueFunction::Spl(proc, _)
            | ValueFunction::BuiltIn(proc, _) => Box::new(proc.parameters.iter()),
        }
    }

    pub const fn local_table(&self) -> &SymbolTable {
        match self {
            ValueFunction::Tac(proc, _)
            | ValueFunction::Spl(proc, _)
            | ValueFunction::BuiltIn(proc, _) => &proc.local_table,
        }
    }
}

#[derive(Clone)]
pub struct BuiltInProc {
    implementation: Rc<BuiltInProcFn>,
}
type BuiltInProcFn = dyn Fn(&[ValueRef<'_>]);
impl BuiltInProc {
    pub fn call(&self, args: &[ValueRef]) {
        (self.implementation)(args);
    }
}
impl Debug for BuiltInProc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltInProc")
            .field(
                "implementation",
                &format_args!(
                    "Rc<BuiltInProcFn({:?})>",
                    std::ptr::from_ref(self.implementation.as_ref())
                ),
            )
            .finish()
    }
}

impl Value<'_> {
    pub fn new_refcell(value: Value) -> ValueRef {
        Rc::new(RefCell::new(value))
    }

    pub fn new_builtin_proc(
        params: impl Iterator<Item = (String, bool)>,
        f: impl Fn(&[ValueRef<'_>]) + 'static,
    ) -> Self {
        Value::Function(ValueFunction::BuiltIn(
            ProcedureEntry {
                local_table: SymbolTable::new(),
                parameters: params
                    .map(|(name, is_reference)| Parameter {
                        name,
                        typ: Type::INT,
                        is_reference,
                    })
                    .collect(),
            },
            BuiltInProc {
                implementation: Rc::new(f),
            },
        ))
    }
}

impl Add for Value<'_> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else {
            unreachable!("{self:?} is not Int");
        };
        let Self::Int(j) = rhs else {
            unreachable!("{rhs:?} is not Int");
        };

        Self::Int(i + j)
    }
}

impl Sub for Value<'_> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else {
            unreachable!("{self:?} is not Int");
        };
        let Self::Int(j) = rhs else {
            unreachable!("{rhs:?} is not Int");
        };

        Self::Int(i - j)
    }
}

impl Mul for Value<'_> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else {
            unreachable!("{self:?} is not Int");
        };
        let Self::Int(j) = rhs else {
            unreachable!("{rhs:?} is not Int")
        };

        Self::Int(i * j)
    }
}

impl Div for Value<'_> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else {
            unreachable!("{self:?} is not Int");
        };
        let Self::Int(j) = rhs else {
            unreachable!("{rhs:?} is not Int");
        };

        Self::Int(i / j)
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        let i = match self {
            Self::Int(i) => *i,
            Self::Bool(b) => i32::from(*b),
            _ => unreachable!(),
        };

        let j = match other {
            Self::Int(j) => *j,
            Self::Bool(b) => i32::from(*b),
            _ => unreachable!(),
        };

        i == j
    }
}

impl PartialOrd for Value<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let i = match self {
            Self::Int(i) => *i,
            Self::Bool(b) => i32::from(*b),
            _ => unreachable!(),
        };

        let j = match other {
            Self::Int(j) => *j,
            Self::Bool(b) => i32::from(*b),
            _ => unreachable!(),
        };

        i.partial_cmp(&j)
    }
}

impl Neg for Value<'_> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let Self::Int(i) = self else { unreachable!() };

        Self::Int(-i)
    }
}
