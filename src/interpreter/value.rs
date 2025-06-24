use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
    rc::Rc,
};

use crate::absyn::{
    absyn::{TypeExpression, Variable},
    parameter_definition::ParameterDefinition,
    procedure_definition::ProcedureDefinition,
};

use super::{environment::Environment, statement_evaluator::eval_var_mut};

#[derive(Clone)]
pub enum Value<'a> {
    Int(i32),
    Bool(bool),
    Array(Vec<Value<'a>>),
    Function(ValueFunction<'a>),
    Ref(ValueRef<'a>),
}

#[derive(Clone)]
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
type BuiltInProcFn = dyn Fn(&[Value]);
impl BuiltInProc {
    pub fn call(&self, args: &[Value]) {
        (self.implementation)(args);
    }
}

impl Value<'_> {
    pub fn new_builtin_proc<const N: usize>(
        params: &[(&str, bool); N],
        f: impl Fn(&[Value]) + 'static,
    ) -> Self {
        Value::Function(ValueFunction::BuiltIn(BuiltInProc {
            implementation: Rc::new(f),
            parameters: params
                .iter()
                .map(|&(name, is_reference)| ParameterDefinition {
                    name: name.to_owned(),
                    type_expression: TypeExpression::NamedTypeExpression("int".to_owned()),
                    is_reference,
                })
                .collect(),
        }))
    }
}

#[derive(Clone)]
pub struct ValueRef<'a> {
    pub var: &'a Variable,
    pub env: Rc<Environment<'a>>,
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

impl Value<'_> {
    pub fn assign(&mut self, new_val: Self) {
        match self {
            Self::Ref(val_ref) => val_ref.assign(&new_val),
            _ => *self = new_val,
        }
    }
}

impl<'a> ValueRef<'a> {
    fn assign(&self, new_val: &Value<'a>) {
        eval_var_mut(self.var, &self.env, &|var| var.assign(new_val.clone()));
    }
}
