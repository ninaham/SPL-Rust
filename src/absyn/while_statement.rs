use super::absyn::{Expression, Statement};

#[derive(Debug)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Statement,
}
