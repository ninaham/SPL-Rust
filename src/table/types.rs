#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    ArrayType(ArrayType),
    PrimitiveType(PrimitiveType),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ArrayType {
    pub base_type: Box<Type>,
    pub size: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PrimitiveType {
    Int,
    Bool,
}
