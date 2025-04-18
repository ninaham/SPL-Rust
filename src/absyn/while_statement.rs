use super::absyn::{Expression, Statement};

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Statement,
}
