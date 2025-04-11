#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    ArrayType(ArrayType),
    PrimitiveType(PrimitiveType),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ArrayType {
    pub base_type: Box<Type>,
    pub size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Int,
    Bool,
}
