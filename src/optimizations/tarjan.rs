use std::collections::{HashMap, HashSet};

use crate::base_blocks::{Block, BlockGraph};

type BlockId = usize;

#[derive(Debug, Default)]
pub struct Scc<'a> {
    pub scc: Vec<Vec<&'a Block>>,
}

//TODO: Umbenennen Bob, Datenstruktur für einzelne SCC: Liste Blöcke und Parent (optional), strong_connect Argumente so, dass Clippy nicht mehr nervt

struct Bob<'a> {
    id: BlockId,
    index: &mut usize,
    index_map: &mut HashMap<BlockId, usize>,
    lowlink_map: &mut HashMap<BlockId, usize>,
    on_stack: &mut HashSet<BlockId>,
    stack: &mut Vec<BlockId>,
    sccs: &mut Vec<Vec<&'a Block>>,
}

impl BlockGraph {
    pub fn tarjan(&self) -> Scc {
        let mut index = 0;
        let mut index_map = HashMap::<BlockId, usize>::new();
        let mut lowlink_map = HashMap::<BlockId, usize>::new();
        let mut on_stack = HashSet::<BlockId>::new();
        let mut stack = Vec::<BlockId>::new();
        let mut sccs = Vec::<Vec<&Block>>::new();

        for id in 0..self.blocks.len() {
            if !index_map.contains_key(&id) {
                let mut bob = Bob {
                    id: &id,
                    index: &mut index,
                    index_map: &mut index_map,
                    lowlink_map: &mut lowlink_map,
                    on_stack: &mut on_stack,
                    stack: &mut stack,
                    sccs: &mut sccs,
                };
                self.strong_connect(bob);
            }
        }

        Scc { scc: sccs }
    }

    fn strong_connect<'a>(&'a self, bob: Bob) {
        bob.index_map.insert(bob.id, bob.index);
        bob.lowlink_map.insert(bob.id, *bob.index);
        *bob.index += 1;

        bob.stack.push(bob.id);
        bob.on_stack.insert(bob.id);

        for next_block in &self.edges[bob.id] {
            if !bob.index_map.contains_key(next_block) {
                let mut bob = Bob {
                    id: *next_block,
                    ..bob
                };
                self.strong_connect(bob);
                let lowlink = bob.lowlink_map[&bob.id].min(bob.lowlink_map[next_block]);
                bob.lowlink_map.insert(bob.id, lowlink);
            } else if bob.on_stack.contains(next_block) {
                let lowlink = bob.lowlink_map[&bob.id].min(bob.index_map[next_block]);
                bob.lowlink_map.insert(bob.id, lowlink);
            }
        }

        if bob.lowlink_map[&bob.id] == bob.index_map[&bob.id] {
            let mut component = Vec::new();
            while let Some(w) = bob.stack.pop() {
                bob.on_stack.remove(&w);
                component.push(&self.blocks[w]);
                if w == bob.id {
                    break;
                }
            }
            bob.sccs.push(component);
        }
    }
}
