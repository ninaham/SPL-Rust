use crate::{
    absyn::{binary_expression::Operator, unary_expression::UnaryOperator},
    table::types::{PrimitiveType, Type},
};

use super::QuadrupelOp;

impl Type {
    pub fn get_byte_size(&self) -> i32 {
        match self {
            Self::ArrayType(a) => a.base_type.get_byte_size() * a.size as i32,
            Self::PrimitiveType(PrimitiveType::Int) => 4,
            _ => panic!("Byte size not defined for {self:?}"),
        }
    }
}

impl From<Operator> for QuadrupelOp {
    fn from(op: Operator) -> Self {
        match op {
            Operator::Add => QuadrupelOp::Add,
            Operator::Sub => QuadrupelOp::Sub,
            Operator::Mul => QuadrupelOp::Mul,
            Operator::Div => QuadrupelOp::Div,
            Operator::Equ => QuadrupelOp::Equ,
            Operator::Neq => QuadrupelOp::Neq,
            Operator::Lst => QuadrupelOp::Lst,
            Operator::Lse => QuadrupelOp::Lse,
            Operator::Grt => QuadrupelOp::Grt,
            Operator::Gre => QuadrupelOp::Gre,
        }
    }
}

impl From<UnaryOperator> for QuadrupelOp {
    fn from(op: UnaryOperator) -> Self {
        match op {
            UnaryOperator::Minus => QuadrupelOp::Sub,
        }
    }
}
