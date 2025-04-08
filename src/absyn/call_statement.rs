use std::collections::LinkedList;

use super::absyn::Expression;

#[derive(Debug)]
pub struct CallStatement {
    pub name: String,
    pub arguments: LinkedList<Expression>,
}
