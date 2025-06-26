use crate::{
    base_blocks::{BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelResult, QuadrupelVar},
    optimizations::{live_variables::LiveVariables, worklist::GetVarIdx},
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
                        .and_then(|var| livar.get_var_idx(var))
                        .is_some_and(|idx| !liveout[idx]);

                    if is_dead {
                        *quad = Quadrupel::EMPTY;
                    } else {
                        for var in vars_from_quad(quad) {
                            if let Some(idx) = livar.get_var_idx(&var) {
                                liveout.set(idx, true);
                            }
                        }
                    }
                }

                Quadrupel::filter_empty(code);
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

impl Quadrupel {
    pub fn filter_empty(quads: &mut Vec<Self>) {
        quads.retain(|quad| quad != &Self::EMPTY);
    }
}
