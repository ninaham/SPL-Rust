use crate::{
    absyn::absyn::Expression,
    table::types::{PrimitiveType, Type},
};

impl Type {
    pub fn is_array(&self) -> bool {
        matches!(self, Self::ArrayType(_))
    }

    pub fn is_bool(&self) -> bool {
        self == &Self::PrimitiveType(PrimitiveType::Bool)
    }

    pub fn is_int(&self) -> bool {
        self == &Self::PrimitiveType(PrimitiveType::Int)
    }
}

impl Expression {
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::VariableExpression(_))
    }
}
