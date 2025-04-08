use super::absyn::{Expression, Variable};

#[derive(Debug)]
pub struct AssignStatement {
    pub target: Variable,
    pub value: Expression,
}
