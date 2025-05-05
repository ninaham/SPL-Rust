use super::aeb::AEBEntry;

use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{
        quad, Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar,
    },
};

impl BlockGraph {
    pub fn common_subexpression_elimination(&mut self) {
        let mut tmp_last_num = self
            .blocks
            .iter()
            .filter_map(|b| match &b.content {
                BlockContent::Code(quads) => Some(quads.iter().filter_map(|q| match q.result {
                    QuadrupelResult::Var(QuadrupelVar::Tmp(n)) => Some(n),
                    _ => None,
                })),
                _ => None,
            })
            .flatten()
            .max()
            .unwrap_or(0);

        let mut tmp_next_num = || -> usize {
            tmp_last_num += 1;
            tmp_last_num
        };

        self.blocks
            .iter_mut()
            .for_each(|b| optimize_block(b, &mut tmp_next_num));
    }
}

fn optimize_block(block: &mut Block, tmp_next_num: &mut impl FnMut() -> usize) {
    let BlockContent::Code(quads) = &mut block.content else {
        return;
    };

    let mut aeb: Vec<AEBEntry> = Vec::new();
    let mut code_new = Vec::<Option<Quadrupel>>::new();

    for quad in quads.iter().cloned() {
        let Some(mut quad) = quad.simplify() else {
            continue;
        };

        match quad.op {
            QuadrupelOp::Add
            | QuadrupelOp::Sub
            | QuadrupelOp::Mul
            | QuadrupelOp::Div
            | QuadrupelOp::Neg => {
                if let Some(entry) = aeb.iter_mut().find(|e| e.cmp(&quad)) {
                    let tmp = entry.tmp.get_or_insert_with(|| {
                        let tmp = QuadrupelVar::Tmp(tmp_next_num());
                        let mut q = entry.quad.clone();
                        q.result = QuadrupelResult::Var(tmp.clone());
                        code_new[entry.pos - 1] = Some(q);
                        code_new[entry.pos] = Some(
                            quad!((:=), (QuadrupelArg::Var(tmp.clone())), _ => entry.quad.result.clone()),
                        );
                        tmp
                    });

                    quad = quad!((:=), (QuadrupelArg::Var(tmp.clone())), _ => quad.result);
                } else {
                    code_new.push(None);
                    aeb.push(AEBEntry::new(quad.clone(), code_new.len()));
                }
            }
            _ => {}
        }

        aeb.retain(|e| quad.result != e.quad.arg1 && quad.result != e.quad.arg2);

        code_new.push(Some(quad));
    }

    *quads = code_new.into_iter().flatten().collect();
}
