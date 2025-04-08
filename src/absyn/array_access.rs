use super::absyn::{Expression, Variable};

pub struct ArrayAccess {
    pub array: Variable,
    pub index: Expression,
}
