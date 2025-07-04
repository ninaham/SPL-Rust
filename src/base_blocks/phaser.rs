use std::collections::HashSet;

use crate::code_gen::quadrupel::Quadrupel;

use super::{Block, BlockGraph, block_start_iter::BlockStartIterator};

/// Phase 1: Identify basic block start indices using a custom iterator.
///
/// Returns an iterator over tuples representing the `(start, end)` indices of basic blocks.
pub const fn phase_1(code: &[Quadrupel]) -> BlockStartIterator<'_> {
    BlockStartIterator::new(code)
}

/// Phase 2: Construct a `BlockGraph` from code and block start indices.
///
/// This function builds a sequence of blocks from the provided block ranges,
/// and links them linearly, starting from a "start" block and ending with a "stop" block.
pub fn phase_2(
    block_starts: impl Iterator<Item = (usize, usize)>,
    code: &[Quadrupel],
) -> BlockGraph {
    let code: Vec<Quadrupel> = code.to_vec();

    let mut graph = BlockGraph::new();
    // Create the start block
    let mut last_id = graph.add_block(Block::new_start(Some("start".to_string())), None);

    // Create one block per identified code segment
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

    // Create the stop block and link it to the last code block
    graph.add_block(Block::new_stop(Some("stop".to_string())), Some(last_id));

    graph
}

/// Phase 3: Analyze control flow to fix the edges in the `BlockGraph`.
///
/// Replaces or adds control flow edges for `goto` and conditional jumps.
pub fn phase_3(mut block_graph: BlockGraph) -> BlockGraph {
    let blocks = block_graph.blocks.clone();

    for (i, b) in blocks.iter().enumerate() {
        // If the block ends with an unconditional GOTO, replace its edge
        if let Some(label) = b.contains_goto() {
            block_graph.edges[i] = HashSet::new();
            let label_block = block_graph
                .label_to_id
                .get(&label)
                .unwrap_or_else(|| panic!("block corresponding to label {label} not found"));
            block_graph.add_edge(i, *label_block);
        }

        // If the block ends with a conditional jump, add an edge to the target label
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
