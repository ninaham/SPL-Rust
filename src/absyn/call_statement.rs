use super::absyn::Expression;

pub struct CallStatement<'a> {
    pub name: String,
    pub arguments: Vec<Expression<'a>>,
}
