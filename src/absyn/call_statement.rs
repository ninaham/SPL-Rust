use std::collections::LinkedList;

use super::absyn::Expression;

pub struct CallStatement {
    pub name: String,
    pub arguments: LinkedList<Expression>,
}
