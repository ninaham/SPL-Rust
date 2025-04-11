use crate::absyn::absyn::{Expression, TypeExpression};

impl TypeExpression {
    const PRIMITIVE_BOOL: Self = Self::NamedTypeExpression("bool".to_owned());
    const PRIMITIVE_INT: Self = Self::NamedTypeExpression("int".to_owned());

    pub fn is_array(&self) -> bool {
        if let Self::ArrayTypeExpression(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_int(&self) -> bool {
        Self::PRIMITIVE_INT.eq(self)
    }

    pub fn is_bool(&self) -> bool {
        Self::PRIMITIVE_BOOL.eq(self)
    }
}

impl Expression {
    pub fn is_variable(&self) -> bool {
        if let Self::VariableExpression(_) = self {
            true
        } else {
            false
        }
    }
}
