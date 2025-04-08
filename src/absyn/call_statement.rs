use super::absyn::Expression;

pub struct CallStatement {
    pub name: String,
    pub arguments: Vec<Expression>,
}
