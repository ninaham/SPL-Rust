use super::absyn::{Expression, Statement};

pub struct IfStatement<'a> {
    pub condition: &'a Expression<'a>,
    pub then_branch: &'a Statement<'a>,
    pub else_branch: Option<&'a Statement<'a>>,
}
