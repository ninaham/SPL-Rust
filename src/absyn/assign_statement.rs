use super::absyn::{Expression, Variable};

#[derive(Debug, Clone)]
pub struct AssignStatement {
    pub target: Variable,
    pub value: Expression,
}
