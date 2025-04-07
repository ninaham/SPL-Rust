use super::absyn::{Expression, Variable};

pub struct ArrayAccess<'a> {
    pub array: &'a Variable<'a>,
    pub index: &'a Expression<'a>,
}
