use crate::{
    absyn::{binary_expression::Operator, unary_expression::UnaryOperator},
    table::types::{PrimitiveType, Type},
};

use super::QuadrupelOp;

impl Type {
    pub fn get_byte_size(&self) -> i32 {
        #[expect(clippy::match_wildcard_for_single_variants)]
        match self {
            Self::ArrayType(a) => a.base_type.get_byte_size() * i32::try_from(a.size).unwrap(),
            Self::PrimitiveType(PrimitiveType::Int) => 4,
            _ => panic!("Byte size not defined for {self:?}"),
        }
    }
}

impl From<Operator> for QuadrupelOp {
    fn from(op: Operator) -> Self {
        match op {
            Operator::Add => Self::Add,
            Operator::Sub => Self::Sub,
            Operator::Mul => Self::Mul,
            Operator::Div => Self::Div,
            Operator::Equ => Self::Equ,
            Operator::Neq => Self::Neq,
            Operator::Lst => Self::Lst,
            Operator::Lse => Self::Lse,
            Operator::Grt => Self::Grt,
            Operator::Gre => Self::Gre,
        }
    }
}

impl QuadrupelOp {
    pub fn inv(self) -> Self {
        match self {
            Self::Equ => Self::Neq,
            Self::Neq => Self::Equ,
            Self::Lst => Self::Gre,
            Self::Lse => Self::Grt,
            Self::Grt => Self::Lse,
            Self::Gre => Self::Lst,
            _ => panic!("{self}: not invertable!"),
        }
    }
}

impl From<UnaryOperator> for QuadrupelOp {
    fn from(op: UnaryOperator) -> Self {
        match op {
            UnaryOperator::Minus => Self::Neg,
        }
    }
}
