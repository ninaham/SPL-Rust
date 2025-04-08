use super::absyn::{Expression, Statement};

pub struct WhileStatement {
    pub condition: Expression,
    pub body: Statement,
}
