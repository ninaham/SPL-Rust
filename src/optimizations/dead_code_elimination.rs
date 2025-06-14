use crate::{
    base_blocks::{BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelResult, QuadrupelVar},
    optimizations::live_variables::LiveVariables,
};

impl BlockGraph {
    pub fn dead_code_elimination(&mut self, livar: &LiveVariables) {
        for (blknum, block) in self.blocks.iter_mut().enumerate() {
            let mut liveout = livar.livout[blknum].clone();

            if let BlockContent::Code(code) = &mut block.content {
                for quad in code.iter_mut().rev() {
                    let res_var = match &quad.result {
                        QuadrupelResult::Var(var) => Some(var),
                        _ => None,
                    };

                    let is_dead = res_var
                        .and_then(|var| livar.vars.iter().position(|v| v == var))
                        .is_some_and(|idx| !liveout[idx]);

                    if is_dead {
                        *quad = Quadrupel::EMPTY;
                    } else {
                        for var in vars_from_quad(quad) {
                            if let Some(idx) = livar.vars.iter().position(|v| v == &var) {
                                liveout.set(idx, true);
                            }
                        }
                    }
                }

                code.retain(|quad| quad != &Quadrupel::EMPTY);
            }
        }
    }
}

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
