use crate::{
    absyn::{absyn::Expression, binary_expression::Operator, unary_expression::UnaryOperator},
    table::types::{PrimitiveType, Type},
};

// Implementations for type checks and constants on the Type enum
impl Type {
    // Constant for the primitive integer type
    pub const INT: Self = Self::PrimitiveType(PrimitiveType::Int);

    // Constant for the primitive boolean type
    pub const BOOL: Self = Self::PrimitiveType(PrimitiveType::Bool);

    // Check if a type is an array type
    pub const fn is_array(&self) -> bool {
        matches!(self, Self::ArrayType(_))
    }

    // Check if a type is boolean (i.e., matches the BOOL constant)
    pub fn is_bool(&self) -> bool {
        self == &Self::BOOL
    }

    // Check if a type is integer (i.e., matches the INT constant)
    pub fn is_int(&self) -> bool {
        self == &Self::INT
    }
}

// Implement utility method for Expression to check if it's a variable expression
impl Expression {
    // Returns true if the expression is a variable (e.g., `x`, `arr[i]`)
    pub const fn is_variable(&self) -> bool {
        matches!(self, Self::VariableExpression(_))
    }
}

// Operator implementation for determining the result type of a binary operation
impl Operator {
    // Returns the resulting type of applying the operator to two operand types
    pub fn result_type(self, left_type: &Type, right_type: &Type) -> Option<Type> {
        // All binary operations defined here require integer operands
        if !left_type.is_int() || !right_type.is_int() {
            return None;
        }

        // Arithmetic operations result in INT
        // Comparison operations result in BOOL
        Some(match self {
            Self::Add | Self::Sub | Self::Mul | Self::Div => Type::INT,
            Self::Equ | Self::Neq | Self::Lst | Self::Lse | Self::Grt | Self::Gre => Type::BOOL,
        })
    }
}

// UnaryOperator implementation for determining the result type of a unary operation
impl UnaryOperator {
    // Returns the resulting type of applying the operator to a single operand type
    pub fn result_type(self, right_type: &Type) -> Option<Type> {
        // All unary operators in this language currently only apply to integers
        if !(right_type.is_int()) {
            return None;
        }

        // The only defined unary operator is negation (minus), which keeps INT type
        Some(match self {
            Self::Minus => Type::INT,
        })
    }
}
