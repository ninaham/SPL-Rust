use crate::interpreter::value::Value;

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

impl Type {
    pub fn default_value<'a>(&self) -> Value<'a> {
        match self {
            Self::ArrayType(array_type) => {
                Value::Array(vec![array_type.base_type.default_value(); array_type.size])
            }
            Self::PrimitiveType(primitive_type) => match primitive_type {
                PrimitiveType::Int => Value::Int(0),
                PrimitiveType::Bool => Value::Bool(false),
            },
        }
    }
}
