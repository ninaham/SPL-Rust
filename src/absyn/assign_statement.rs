use super::absyn::{Expression, Variable};

pub struct AssignStatement<'a> {
    pub target: &'a Variable<'a>,
    pub value: &'a Expression<'a>,
}
