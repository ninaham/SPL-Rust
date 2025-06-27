use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt::Debug,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

use crate::absyn::{
    absyn::TypeExpression, parameter_definition::ParameterDefinition,
    procedure_definition::ProcedureDefinition,
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
    Spl(&'a ProcedureDefinition),
    BuiltIn(BuiltInProc),
}
impl ValueFunction<'_> {
    pub fn parameters(&self) -> Box<dyn Iterator<Item = &ParameterDefinition> + '_> {
        match self {
            ValueFunction::Spl(proc) => Box::new(proc.parameters.iter()),
            ValueFunction::BuiltIn(proc) => Box::new(proc.parameters.iter()),
        }
    }
}

#[derive(Clone)]
pub struct BuiltInProc {
    implementation: Rc<BuiltInProcFn>,
    parameters: Vec<ParameterDefinition>,
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
            .field("parameters", &self.parameters)
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
        Value::Function(ValueFunction::BuiltIn(BuiltInProc {
            implementation: Rc::new(f),
            parameters: params
                .map(|(name, is_reference)| ParameterDefinition {
                    name,
                    type_expression: TypeExpression::NamedTypeExpression("int".to_owned()),
                    is_reference,
                })
                .collect(),
        }))
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
