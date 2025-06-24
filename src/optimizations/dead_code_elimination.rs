use crate::{
    base_blocks::{BlockContent, BlockGraph},
    code_gen::quadrupel::{
        quad, quad_match, Quadrupel, QuadrupelArg, QuadrupelResult, QuadrupelVar,
    },
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

    pub fn dead_block_elimination(&mut self) {
        for (block_id, block) in self.blocks.iter().enumerate() {
            if let BlockContent::Code(quads) = &block.content {
                let quad_end = quads.last();
                let mut next_blocks = Vec::new();

                if let Some(quad_match!(=> QuadrupelResult::Label(label))) = quad_end {
                    next_blocks.push(self.label_to_id(label));
                } else {
                    next_blocks.push(block_id + 1);

                    if let Some(quad_match!(_, _, _ => QuadrupelResult::Label(label))) = quad_end {
                        next_blocks.push(self.label_to_id(label));
                    }
                }

                self.edges
                    .get_mut(block_id)
                    .unwrap()
                    .retain(|id| next_blocks.contains(id));
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
