use crate::code_gen::Quadrupel;

use super::{Block, BlockContent, BlockGraph};

pub fn phase_2(block_starts: impl Iterator<Item = usize>, code: &[Quadrupel]) -> BlockGraph {
    let mut last = 0;
    let code: Vec<Quadrupel> = code.to_vec();

    let mut graph = BlockGraph::new();
    graph.add_block(
        Block::new(Some("start".to_string()), BlockContent::Start),
        None,
    );
    block_starts
        .into_iter()
        .enumerate()
        .for_each(|(parent, split)| {
            let label = match code[last].op {
                crate::code_gen::QuadrupelOp::Default => Some(code[last].result.to_string()),
                _ => None,
            };
            graph.add_block(
                Block::new(label, super::BlockContent::Code(code[last..split].to_vec())),
                Some(parent),
            );
            last = split;
        });
    graph.add_block(
        Block::new(Some("stop".to_string()), BlockContent::Stop),
        Some(graph.blocks.len()),
    );
    graph
}
