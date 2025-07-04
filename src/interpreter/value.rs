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

// A reference to a value in the interpreter, allowing for shared ownership and mutable access.
pub type ValueRef<'a> = Rc<RefCell<Value<'a>>>;

// Represents a value in the interpreter, which can be an integer, boolean, array, or function.
#[derive(Clone, Debug)]
pub enum Value<'a> {
    Int(i32),
    Bool(bool),
    Array(Vec<ValueRef<'a>>),
    Function(ValueFunction<'a>),
}

// Represents a function in the interpreter, which can be a procedure with a body (Spl), a procedure with a block graph (Tac), or a built-in procedure.
#[derive(Clone, Debug)]
pub enum ValueFunction<'a> {
    #[expect(clippy::linkedlist)]
    Spl(ProcedureEntry, &'a LinkedList<Statement>),
    Tac(ProcedureEntry, &'a BlockGraph),
    BuiltIn(ProcedureEntry, BuiltInProc),
}
impl ValueFunction<'_> {
    // Returns the procedure entry associated with the function, regardless of its type.
    pub const fn entry(&self) -> &ProcedureEntry {
        match self {
            ValueFunction::Tac(proc, _)
            | ValueFunction::Spl(proc, _)
            | ValueFunction::BuiltIn(proc, _) => proc,
        }
    }
}

// Represents a built-in procedure.
#[derive(Clone)]
pub struct BuiltInProc {
    implementation: Rc<BuiltInProcFn>,
}

// A Built-in procedure is a rust function that takes ValueRef arguments.
type BuiltInProcFn = dyn Fn(&[ValueRef<'_>]);
impl BuiltInProc {
    // calls the built-in procedure with the provided arguments.
    pub fn call(&self, args: &[ValueRef]) {
        (self.implementation)(args);
    }
}

// Implements the Debug trait for BuiltInProc to print it for debug purposes.
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
    // Flattens an array value by recursively flattening any nested arrays within it. We need this to ensure that arrays can be treated as flat arrays of values in the TAC interpreter.
    pub fn flatten_value(&self) -> Self {
        if let Value::Array(ref_cells) = self {
            let mut arr = vec![];
            for v in ref_cells {
                if let Value::Array(a) = v.borrow().flatten_value() {
                    for x in a {
                        arr.push(x);
                    }
                } else {
                    arr.push(v.clone());
                }
            }
            Value::Array(arr)
        } else {
            self.clone()
        }
    }
    // Creates a new ValueRef containing a RefCell with the given value.
    pub fn new_refcell(value: Value) -> ValueRef {
        Rc::new(RefCell::new(value))
    }

    // Creates a new built-in procedure with the given parameters and a rust function, that implements it.
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

// Implements arithmetic operations for Value, allowing addition, subtraction, multiplication, and division of integer values.
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

// Implements the PartialEq and PartialOrd traits for Value, allowing comparison of integer and boolean values.
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

// Implements the Neg trait for Value, allowing negation of integer values.
impl Neg for Value<'_> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let Self::Int(i) = self else { unreachable!() };

        Self::Int(-i)
    }
}
