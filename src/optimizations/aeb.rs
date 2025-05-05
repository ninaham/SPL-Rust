use crate::code_gen::quadrupel::{Quadrupel, QuadrupelOp, QuadrupelVar};

pub(super) struct AEBEntry {
    pub quad: Quadrupel,
    pub pos: usize,
    pub tmp: Option<QuadrupelVar>,
}

impl AEBEntry {
    pub fn new(quad: Quadrupel, pos: usize) -> Self {
        Self {
            quad,
            pos,
            tmp: None,
        }
    }

    pub fn cmp(&self, other: &Quadrupel) -> bool {
        match self.quad.op {
            QuadrupelOp::Add | QuadrupelOp::Mul | QuadrupelOp::Neg => {
                (self.quad.arg1 == other.arg1 && self.quad.arg2 == other.arg2)
                    || (self.quad.arg1 == other.arg2 && self.quad.arg2 == other.arg1)
            }
            QuadrupelOp::Sub | QuadrupelOp::Div => {
                self.quad.arg1 == other.arg1 && self.quad.arg2 == other.arg2
            }
            QuadrupelOp::ArrayLoad => self.quad.arg1 == other.arg1 && self.quad.arg2 == other.arg2,
            _ => false,
        }
    }
}
