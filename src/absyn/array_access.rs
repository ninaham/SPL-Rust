use super::absyn::{Expression, Variable};

#[derive(Debug)]
pub struct ArrayAccess {
    pub array: Variable,
    pub index: Expression,
}
