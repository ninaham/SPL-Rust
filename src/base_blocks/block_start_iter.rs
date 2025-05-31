use std::ops::ControlFlow;

use crate::code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult};

pub struct BlockStartIterator<'a> {
    code: &'a [Quadrupel],
    prev_index: usize,
}

impl<'a> BlockStartIterator<'a> {
    pub const fn new(code: &'a [Quadrupel]) -> Self {
        Self {
            code,
            prev_index: 0,
        }
    }
}

impl Iterator for BlockStartIterator<'_> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.prev_index >= self.code.len() {
            return None;
        }

        let last = self.prev_index;

        let mut iter = self.code.iter().enumerate().skip(last);

        self.prev_index = iter
            .try_for_each(|(i, quad)| {
                if quad.op.is_any_jump() {
                    return ControlFlow::Break(i + 1);
                }

                if matches!(
                    quad,
                    Quadrupel {
                        op: QuadrupelOp::Default,
                        arg1: QuadrupelArg::Empty,
                        arg2: QuadrupelArg::Empty,
                        result: QuadrupelResult::Label(_),
                    }
                ) && i > last
                {
                    return ControlFlow::Break(i);
                }

                ControlFlow::Continue(())
            })
            .break_value()
            .unwrap_or(self.code.len());

        Some((last, self.prev_index))
    }
}
