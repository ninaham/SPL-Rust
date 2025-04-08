use super::absyn::{Expression, Variable};

pub struct AssignStatement {
    pub target: Variable,
    pub value: Expression,
}
