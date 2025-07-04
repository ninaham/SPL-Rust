use crate::{
    base_blocks::{BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelResult, QuadrupelVar},
    optimizations::{live_variables::LiveVariables, worklist::GetVarIdx},
};

impl BlockGraph {
    // Performs dead code elimination on the block graph using live variable analysis.
    pub fn dead_code_elimination(&mut self, livar: &LiveVariables) {
        for (blknum, block) in self.blocks.iter_mut().enumerate() {
            // Clone the set of live variables at the end of the block.
            let mut liveout = livar.livout[blknum].clone();

            // Only process blocks that contain code.
            if let BlockContent::Code(code) = &mut block.content {
                // Iterate the code in reverse (from end to start).
                for quad in code.iter_mut().rev() {
                    // Try to get the result variable of the current instruction.
                    let res_var = match &quad.result {
                        QuadrupelResult::Var(var) => Some(var),
                        _ => None,
                    };

                    // A variable is dead if it is not live after this point.
                    let is_dead = res_var
                        .and_then(|var| livar.get_var_idx(var))
                        .is_some_and(|idx| !liveout[idx]);

                    if is_dead {
                        // Replace the instruction with an EMPTY placeholder.
                        *quad = Quadrupel::EMPTY;
                    } else {
                        // Otherwise, mark used variables as live.
                        for var in vars_from_quad(quad) {
                            if let Some(idx) = livar.get_var_idx(&var) {
                                liveout.set(idx, true);
                            }
                        }
                    }
                }

                // Remove all EMPTY placeholders from the code.
                Quadrupel::filter_empty(code);
            }
        }
    }
}

// Extracts all variable arguments used in a quadruple.
fn vars_from_quad(quad: &Quadrupel) -> Vec<QuadrupelVar> {
    let mut vars = Vec::new();

    if let QuadrupelArg::Var(v) = &quad.arg1 {
        vars.push(v.clone());
    }

    if let QuadrupelArg::Var(v) = &quad.arg2 {
        vars.push(v.clone());
    }

    vars
}

impl Quadrupel {
    // Removes all EMPTY quadruples from a vector of instructions.
    pub fn filter_empty(quads: &mut Vec<Self>) {
        quads.retain(|quad| quad != &Self::EMPTY);
    }
}
