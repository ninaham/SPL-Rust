use std::ops::ControlFlow;

use crate::code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult};

/// Iterator that finds the start and end indices of basic blocks in a list of quadruples.
pub struct BlockStartIterator<'a> {
    /// A slice of quadruples to analyze
    code: &'a [Quadrupel],
    /// Index of the last block's starting point
    prev_index: usize,
}

impl<'a> BlockStartIterator<'a> {
    /// Create a new `BlockStartIterator` for a given list of quadruples
    pub const fn new(code: &'a [Quadrupel]) -> Self {
        Self {
            code,
            prev_index: 0,
        }
    }
}

impl Iterator for BlockStartIterator<'_> {
    /// Each item is a tuple of (start_index, end_index) representing a basic block
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // If we've already reached the end, return None to stop iteration
        if self.prev_index >= self.code.len() {
            return None;
        }

        // Save the current block's start index
        let last = self.prev_index;

        // Skip to the current position in the code
        let mut iter = self.code.iter().enumerate().skip(last);

        // Try to find the end of the current block
        self.prev_index = iter
            .try_for_each(|(i, quad)| {
                // If the operation is any kind of jump, end the block after this instruction
                if quad.op.is_any_jump() {
                    return ControlFlow::Break(i + 1);
                }

                // If we encounter a label (used as jump target) after the current start,
                // we consider this a new block start and break here.
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

                // Otherwise, continue checking
                ControlFlow::Continue(())
            })
            // Use the break value as the new block end, or the end of code if none found
            .break_value()
            .unwrap_or(self.code.len());

        // Return the current block range
        Some((last, self.prev_index))
    }
}
