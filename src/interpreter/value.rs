use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

use crate::absyn::procedure_definition::ProcedureDefinition;

#[derive(Clone)]
pub enum Value<'a> {
    Int(i32),
    Bool(bool),
    Array(Vec<Value<'a>>),
    Function(ValueFunction<'a>),
}

#[derive(Clone)]
pub enum ValueFunction<'a> {
    Spl(&'a ProcedureDefinition),
    BuiltIn(Rc<dyn Fn(&[Value])>),
}

impl Add for Value<'_> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else { unreachable!() };
        let Self::Int(j) = rhs else { unreachable!() };

        Self::Int(i + j)
    }
}

impl Sub for Value<'_> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else { unreachable!() };
        let Self::Int(j) = rhs else { unreachable!() };

        Self::Int(i - j)
    }
}

impl Mul for Value<'_> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else { unreachable!() };
        let Self::Int(j) = rhs else { unreachable!() };

        Self::Int(i * j)
    }
}

impl Div for Value<'_> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let Self::Int(i) = self else { unreachable!() };
        let Self::Int(j) = rhs else { unreachable!() };

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
