#![expect(dead_code)]
use crate::code_gen::quadrupel::{Quadrupel, QuadrupelOp, QuadrupelVar};

pub(super) struct AEBEntry {
    quad: Quadrupel,
    pos: usize,
    tmp: Option<QuadrupelVar>,
}

impl AEBEntry {
    fn cmp(&self, other: &Quadrupel) -> bool {
        match self.quad.op {
            QuadrupelOp::Add | QuadrupelOp::Mul | QuadrupelOp::Neg => {
                (self.quad.arg1 == other.arg1 && self.quad.arg2 == other.arg2)
                    || (self.quad.arg1 == other.arg2 && self.quad.arg2 == other.arg1)
            }
            _ => false,
        }
    }
}
