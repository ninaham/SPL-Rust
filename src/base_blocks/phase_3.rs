use std::collections::HashSet;

use super::{BlockContent, BlockGraph};

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
