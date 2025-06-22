use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
};

#[derive(Clone)]
pub enum Value {
    Int(i32),
    Bool(bool),
    Array(Box<[Value]>),
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        Self::Int(i + j)
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        Self::Int(i - j)
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        Self::Int(i * j)
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        let j = match rhs {
            Self::Int(j) => j,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        Self::Int(i / j)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let i = match self {
            Self::Int(i) => *i,
            Self::Bool(b) => i32::from(*b),
            Self::Array(_) => unreachable!(),
        };

        let j = match other {
            Self::Int(j) => *j,
            Self::Bool(b) => i32::from(*b),
            Self::Array(_) => unreachable!(),
        };

        i == j
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let i = match self {
            Self::Int(i) => *i,
            Self::Bool(b) => i32::from(*b),
            Self::Array(_) => unreachable!(),
        };

        let j = match other {
            Self::Int(j) => *j,
            Self::Bool(b) => i32::from(*b),
            Self::Array(_) => unreachable!(),
        };

        i.partial_cmp(&j)
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let i = match self {
            Self::Int(i) => i,
            Self::Bool(_) | Self::Array(_) => unreachable!(),
        };

        Self::Int(-i)
    }
}
