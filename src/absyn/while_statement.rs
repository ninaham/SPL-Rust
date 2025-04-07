use super::absyn::{Expression, Statement};

pub struct WhileStatement<'a> {
    pub condition: &'a Expression<'a>,
    pub body: &'a Statement<'a>,
}
