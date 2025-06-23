use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::absyn::procedure_definition::ProcedureDefinition;

#[derive(Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Array(Box<[Value]>),
    Function(Box<ProcedureDefinition>),
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            _ => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            _ => unreachable!(),
        };

        Self::Int(i + j)
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            _ => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            _ => unreachable!(),
        };

        Self::Int(i - j)
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            _ => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            _ => unreachable!(),
        };

        Self::Int(i * j)
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            _ => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            _ => unreachable!(),
        };

        Self::Int(i / j)
    }
}

impl PartialEq for Value {
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

impl PartialOrd for Value {
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

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            _ => unreachable!(),
        };

        Self::Int(-i)
    }
}
