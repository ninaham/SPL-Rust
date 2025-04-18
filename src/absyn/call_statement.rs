use std::collections::LinkedList;

use super::absyn::Expression;

#[derive(Debug, Clone)]
pub struct CallStatement {
    pub name: String,
    pub arguments: LinkedList<Expression>,
}
