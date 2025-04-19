use std::ops::ControlFlow;

use crate::code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult};

pub struct BlockStartIterator<'a> {
    code: &'a [Quadrupel],
    prev_index: Option<usize>,
}

impl<'a> BlockStartIterator<'a> {
    pub fn new(code: &'a [Quadrupel]) -> Self {
        Self {
            code,
            prev_index: None,
        }
    }
}

impl Iterator for BlockStartIterator<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if match self.prev_index {
            Some(i) => i >= self.code.len(),
            None => self.code.is_empty(),
        } {
            return None;
        }

        self.prev_index = match self.prev_index {
            None => Some(0),
            Some(last) => {
                let mut iter = self.code.iter().enumerate().skip(last);

                if let ControlFlow::Break(idx) = iter.try_for_each(|(i, quad)| {
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
                    ) {
                        return ControlFlow::Break(i);
                    }

                    ControlFlow::Continue(())
                }) {
                    Some(idx)
                } else {
                    None
                }
            }
        };

        self.prev_index
    }
}
