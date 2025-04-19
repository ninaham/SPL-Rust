use std::collections::HashSet;

use crate::code_gen::quadrupel::Quadrupel;

use super::{block_start_iter::BlockStartIterator, Block, BlockContent, BlockGraph};

pub fn phase_1(code: &[Quadrupel]) -> BlockStartIterator {
    BlockStartIterator::new(code)
}

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
                crate::code_gen::quadrupel::QuadrupelOp::Default => {
                    Some(code[last].result.to_string())
                }
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

pub fn phase_3(mut block_graph: BlockGraph) -> BlockGraph {
    let blocks = block_graph.blocks.clone();
    for (i, b) in blocks.iter().enumerate() {
        if let Some(label) = b.contains_goto() {
            block_graph.edges[i] = HashSet::new();
            let label_block = match block_graph.label_to_id.get(&label) {
                Some(l) => l,
                None => panic!("block corresponding to label {} not found", label),
            };
            block_graph.add_edge(i, *label_block);
        }
        if let Some(label) = b.contains_if() {
            let label_block = match block_graph.label_to_id.get(&label) {
                Some(l) => l,
                None => panic!("block corresponding to label {} not found", label),
            };
            block_graph.add_edge(i, *label_block);
        }
    }

    block_graph
}
