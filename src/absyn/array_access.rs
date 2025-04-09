use super::absyn::{Expression, Variable};

#[derive(Debug, Clone)]
pub struct ArrayAccess {
    pub array: Variable,
    pub index: Expression,
}
