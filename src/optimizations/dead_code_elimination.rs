use super::{super::base_blocks::*, live_variables::LiveVariables};
use crate::code_gen::quadrupel::{QuadrupelOp, QuadrupelResult};
use anyhow::Error;

pub fn dead_code_elimination(
    graph: &BlockGraph,
    livar: &LiveVariables,
) -> Result<BlockGraph, Error> {
    println!("defs: {:?}", &livar.defs);
    let new_blocks = graph
        .blocks
        .iter()
        .enumerate()
        .map(|(blknum, block)| {
            let liveout = &livar.livout[blknum];
            println!("livout: {}", liveout);
            let new_content = match &block.content {
                BlockContent::Start | BlockContent::Stop => block.content.clone(),

                BlockContent::Code(code) => BlockContent::Code(
                    code.iter()
                        .rev()
                        .filter_map(|quad| {
                            let var = match &quad.result {
                                QuadrupelResult::Var(var) => Some(var),
                                _ => None,
                            };

                            let mut is_dead: bool = false;

                            if let Some(var) = var {
                                if let Some(idx) =
                                    &livar.defs.iter().position(|def| def.var == *var)
                                {
                                    is_dead = !liveout[*idx];
                                    println!("idx: {}, is_dead: {}", idx, is_dead);
                                }
                            }

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

                            if is_dead && is_safe_to_remove {
                                None
                            } else {
                                Some(quad.clone())
                            }
                        })
                        .rev()
                        .collect::<Vec<_>>(),
                ),
            };

            Block {
                label: block.label.clone(),
                content: new_content,
                ..block.clone()
            }
        })
        .collect();

    Ok(BlockGraph {
        blocks: new_blocks,
        ..graph.clone()
    })
}
