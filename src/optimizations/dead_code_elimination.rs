use super::{super::base_blocks::*, live_variables::LiveVariables};
use crate::code_gen::quadrupel::QuadrupelOp;
use anyhow::Error;

pub fn dead_code_elimination(
    graph: &BlockGraph,
    livar: &LiveVariables,
) -> Result<BlockGraph, Error> {
    let new_blocks = graph
        .blocks
        .iter()
        .map(|block| {
            let new_content = match &block.content {
                BlockContent::Start | BlockContent::Stop => block.content.clone(),

                BlockContent::Code(code) => BlockContent::Code(
                    code.iter()
                        .enumerate()
                        .rev()
                        .filter_map(|(i, quad)| {
                            let liveout = &livar.livout[i];
                            let defs = &livar.def[i];

                            let is_dead = defs.iter().zip(liveout).all(|(d, l)| !(*d && *l));

                            let is_safe_to_remove = matches!(
                                quad.op,
                                QuadrupelOp::Assign
                                    | QuadrupelOp::ArrayLoad
                                    | QuadrupelOp::ArrayStore
                                    | QuadrupelOp::Neq
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
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect(),
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
