use crate::{
    absyn::{absyn::Expression, binary_expression::Operator, unary_expression::UnaryOperator},
    table::types::{PrimitiveType, Type},
};

impl Type {
    const INT: Self = Self::PrimitiveType(PrimitiveType::Int);
    const BOOL: Self = Self::PrimitiveType(PrimitiveType::Bool);

    pub fn is_array(&self) -> bool {
        matches!(self, Self::ArrayType(_))
    }

    pub fn is_bool(&self) -> bool {
        self == &Self::BOOL
    }

    pub fn is_int(&self) -> bool {
        self == &Self::INT
    }
}

impl Expression {
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::VariableExpression(_))
    }
}

impl Operator {
    pub fn result_type(&self, left_type: &Type, right_type: &Type) -> Option<Type> {
        if !left_type.is_int() || !right_type.is_int() {
            return None;
        }

        Some(match self {
            Self::Add | Self::Sub | Self::Mul | Self::Div => Type::INT,
            Self::Equ | Self::Neq | Self::Lst | Self::Lse | Self::Grt | Self::Gre => Type::BOOL,
        })
    }
}

impl UnaryOperator {
    pub fn result_type(&self, right_type: &Type) -> Option<Type> {
        if !(right_type.is_int()) {
            return None;
        }

        Some(match self {
            Self::Minus => Type::INT,
        })
    }
}
