#![allow(clippy::too_many_arguments)]
use std::collections::{HashMap, HashSet};

use crate::base_blocks::{BlockGraph, BlockId};

pub type Scc = Vec<Vec<BlockId>>;

impl BlockGraph {
    pub fn tarjan(&mut self) -> &Scc {
        let mut index = 0;
        let mut index_map = HashMap::new();
        let mut lowlink_map = HashMap::new();
        let mut on_stack = HashSet::new();
        let mut stack = Vec::new();
        let mut sccs = Vec::new();

        for id in 0..self.blocks.len() {
            if !index_map.contains_key(&id) {
                self.strong_connect(
                    id,
                    &mut index,
                    &mut index_map,
                    &mut lowlink_map,
                    &mut on_stack,
                    &mut stack,
                    &mut sccs,
                );
            }
        }

        self.scc.insert(sccs)
    }

    fn strong_connect(
        &self,
        id: BlockId,
        index: &mut BlockId,
        index_map: &mut HashMap<BlockId, usize>,
        lowlink_map: &mut HashMap<BlockId, usize>,
        on_stack: &mut HashSet<BlockId>,
        stack: &mut Vec<BlockId>,
        sccs: &mut Vec<Vec<BlockId>>,
    ) {
        index_map.insert(id, *index);
        lowlink_map.insert(id, *index);
        *index += 1;

        stack.push(id);
        on_stack.insert(id);

        for next_block in &self.edges[id] {
            if !index_map.contains_key(next_block) {
                self.strong_connect(
                    *next_block,
                    index,
                    index_map,
                    lowlink_map,
                    on_stack,
                    stack,
                    sccs,
                );
                let lowlink = lowlink_map[&id].min(lowlink_map[next_block]);
                lowlink_map.insert(id, lowlink);
            } else if on_stack.contains(next_block) {
                let lowlink = lowlink_map[&id].min(index_map[next_block]);
                lowlink_map.insert(id, lowlink);
            }
        }

        if lowlink_map[&id] == index_map[&id] {
            let mut component = Vec::new();
            while let Some(w) = stack.pop() {
                on_stack.remove(&w);
                component.push(w);
                if w == id {
                    break;
                }
            }

            if component.len() > 1 {
                // our loops always have more than one node
                sccs.push(component);
            }
        }
    }
}
