#[derive(Debug)]
pub enum Type {
    ArrayType(ArrayType),
    PrimitiveType(PrimitiveType),
}

#[derive(Debug)]
pub struct ArrayType {
    pub base_type: Box<Type>,
    pub size: usize,
}

#[derive(Debug)]
pub enum PrimitiveType {
    Int,
    Bool,
}
