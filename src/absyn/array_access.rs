use crate::table::types::ArrayType;

use super::absyn::{Expression, Variable};

#[derive(Debug, Clone)]
pub struct ArrayAccess {
    pub array: Variable,
    pub index: Expression,
    pub typ: Option<ArrayType>,
}
