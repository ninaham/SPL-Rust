#![allow(clippy::too_many_arguments)]

use std::{
    collections::{HashMap, HashSet},
    fmt,
    ops::RangeBounds,
};

use crate::base_blocks::{BlockGraph, BlockId};

/// Represents a strongly connected component (SCC) of basic blocks.
#[derive(Clone)]
pub struct Scc {
    pub nodes: Vec<BlockId>,       // The blocks in this SCC
    pub parent_idx: Option<usize>, // Index of the parent SCC in the hierarchy
    pub children_idx: Vec<usize>,  // Indices of child SCCs
}

impl Scc {
    /// Create a new SCC from a list of nodes and optional parent index.
    const fn new(nodes: Vec<BlockId>, parent_idx: Option<usize>) -> Self {
        Self {
            nodes,
            parent_idx,
            children_idx: Vec::new(),
        }
    }
}

impl BlockGraph {
    /// Entry point for Tarjan's algorithm.
    /// Finds and stores SCCs in the control flow graph.
    pub fn tarjan(&mut self) -> &Vec<Scc> {
        let mut sccs = Vec::new();
        self.tarjan_internal(&mut sccs, 0..self.blocks.len(), None);
        self.sccs.insert(sccs)
    }

    /// Runs Tarjan's algorithm on a subgraph, defined by a range.
    fn tarjan_internal(
        &self,
        sccs: &mut Vec<Scc>,
        subgraph: impl RangeBounds<BlockId> + Clone + IntoIterator<Item = BlockId>,
        parent: Option<usize>,
    ) {
        let mut index = 0;
        let mut index_map = HashMap::new(); // Maps node to its index in DFS traversal
        let mut lowlink_map = HashMap::new(); // Maps node to the lowest index reachable
        let mut on_stack = HashSet::new(); // Tracks nodes currently on the stack
        let mut stack = Vec::new(); // DFS stack

        // Start DFS from each unvisited node in the subgraph
        for id in subgraph.clone() {
            if !index_map.contains_key(&id) {
                self.strong_connect(
                    id,
                    &mut index,
                    &mut index_map,
                    &mut lowlink_map,
                    &mut on_stack,
                    &mut stack,
                    sccs,
                    &subgraph,
                    parent,
                );
            }
        }
    }

    /// Recursive function implementing the core of Tarjan's algorithm.
    fn strong_connect(
        &self,
        id: BlockId,
        index: &mut BlockId,
        index_map: &mut HashMap<BlockId, usize>,
        lowlink_map: &mut HashMap<BlockId, usize>,
        on_stack: &mut HashSet<BlockId>,
        stack: &mut Vec<BlockId>,
        sccs: &mut Vec<Scc>,
        subgraph: &impl RangeBounds<BlockId>,
        parent: Option<usize>,
    ) {
        // Assign DFS index and lowlink
        index_map.insert(id, *index);
        lowlink_map.insert(id, *index);
        *index += 1;

        stack.push(id);
        on_stack.insert(id);

        // Traverse successors within the subgraph
        for next_block in self.edges[id].iter().filter(|i| subgraph.contains(i)) {
            if !index_map.contains_key(next_block) {
                // Recursively visit unvisited successors
                self.strong_connect(
                    *next_block,
                    index,
                    index_map,
                    lowlink_map,
                    on_stack,
                    stack,
                    sccs,
                    subgraph,
                    parent,
                );
                let lowlink = lowlink_map[&id].min(lowlink_map[next_block]);
                lowlink_map.insert(id, lowlink);
            } else if on_stack.contains(next_block) {
                // If successor is on the stack, it's part of the current SCC
                let lowlink = lowlink_map[&id].min(index_map[next_block]);
                lowlink_map.insert(id, lowlink);
            }
        }

        // If `id` is a root node, pop the stack to form an SCC
        if lowlink_map[&id] == index_map[&id] {
            let mut component = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack.remove(&w);
                component.push(w);
                if w == id {
                    break;
                }
            }

            // Only keep loops with more than one node as SCCs
            if component.len() > 1 {
                component.reverse();
                component.sort_unstable();
                let min = *component.first().unwrap();
                let max = *component.last().unwrap();

                sccs.push(Scc::new(component, parent));
                let component_idx = sccs.len() - 1;

                // Recursively apply Tarjan's algorithm to the inner blocks between min and max
                #[expect(clippy::range_minus_one)]
                self.tarjan_internal(sccs, (min + 1)..=(max - 1), Some(component_idx));

                // Register child SCCs
                let sccs_len = sccs.len();
                sccs[component_idx]
                    .children_idx
                    .extend((component_idx + 1)..sccs_len);
            }
        }
    }
}

impl fmt::Debug for Scc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.nodes)
    }
}
