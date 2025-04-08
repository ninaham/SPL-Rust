use super::absyn::{Expression, Statement};

pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Statement,
    pub else_branch: Option<Statement>,
}
