use std::collections::HashSet;

use crate::code_gen::quadrupel::Quadrupel;

use super::{Block, BlockGraph, block_start_iter::BlockStartIterator};

pub const fn phase_1(code: &[Quadrupel]) -> BlockStartIterator<'_> {
    BlockStartIterator::new(code)
}

pub fn phase_2(
    block_starts: impl Iterator<Item = (usize, usize)>,
    code: &[Quadrupel],
) -> BlockGraph {
    let code: Vec<Quadrupel> = code.to_vec();

    let mut graph = BlockGraph::new();
    let mut last_id = graph.add_block(Block::new_start(Some("start".to_string())), None);
    block_starts.for_each(|(start, end)| {
        let label = match code[start].op {
            crate::code_gen::quadrupel::QuadrupelOp::Default => {
                Some(code[start].result.to_string())
            }
            _ => None,
        };
        last_id = graph.add_block(
            Block::new_code(label, code[start..end].to_vec()),
            Some(last_id),
        );
    });
    graph.add_block(Block::new_stop(Some("stop".to_string())), Some(last_id));
    graph
}

pub fn phase_3(mut block_graph: BlockGraph) -> BlockGraph {
    let blocks = block_graph.blocks.clone();
    for (i, b) in blocks.iter().enumerate() {
        if let Some(label) = b.contains_goto() {
            block_graph.edges[i] = HashSet::new();
            let label_block = block_graph
                .label_to_id
                .get(&label)
                .unwrap_or_else(|| panic!("block corresponding to label {label} not found"));
            block_graph.add_edge(i, *label_block);
        }
        if let Some(label) = b.contains_if() {
            let label_block = block_graph
                .label_to_id
                .get(&label)
                .unwrap_or_else(|| panic!("block corresponding to label {label} not found"));
            block_graph.add_edge(i, *label_block);
        }
    }

    block_graph
}
