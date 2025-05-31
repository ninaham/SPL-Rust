use super::{super::base_blocks::*, live_variables::LiveVariables};
use crate::code_gen::quadrupel::{
    Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar,
};
use anyhow::Error;

pub fn dead_code_elimination(graph: &BlockGraph, livar: &LiveVariables) -> BlockGraph {
    let new_blocks = graph
        .blocks
        .iter()
        .enumerate()
        .map(|(blknum, block)| {
            let mut liveout = livar.livout[blknum].clone();

            let new_content = match &block.content {
                BlockContent::Start | BlockContent::Stop => block.content.clone(),
                BlockContent::Code(code) => {
                    let mut new_code = Vec::new();

                    code.iter().rev().for_each(|quad| {
                        let res_var = match &quad.result {
                            QuadrupelResult::Var(var) => Some(var),
                            _ => None,
                        };

                        let is_dead = res_var
                            .and_then(|var| livar.defs.iter().position(|def| def.var == *var))
                            .map_or(false, |idx| !liveout[idx]);

                        let is_safe_to_remove = matches!(
                            quad.op,
                            QuadrupelOp::Assign
                                | QuadrupelOp::ArrayLoad
                                | QuadrupelOp::ArrayStore
                                | QuadrupelOp::Neg
                                | QuadrupelOp::Add
                                | QuadrupelOp::Sub
                                | QuadrupelOp::Mul
                                | QuadrupelOp::Div
                        );

                        if !(is_dead && is_safe_to_remove) {
                            for var in vars_from_quad(quad) {
                                if let Some(idx) = livar.defs.iter().position(|def| def.var == var)
                                {
                                    liveout.set(idx, true);
                                }
                            }
                            new_code.push(quad.clone());
                        }
                    });
                    BlockContent::Code(new_code.into_iter().rev().collect())
                }
            };

            Block {
                label: block.label.clone(),
                content: new_content,
                ..block.clone()
            }
        })
        .collect();

    BlockGraph {
        blocks: new_blocks,
        ..graph.clone()
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
