use std::collections::{HashMap, HashSet};

use phaser::{phase_1, phase_2, phase_3};

use crate::{
    code_gen::quadrupel::{Quadrupel, QuadrupelOp},
    optimizations::reaching_expressions::Definition,
};

mod block_start_iter;
mod dot_graph;
mod phaser;
mod utils;

type BlockId = usize;

#[derive(Debug, Clone)]
pub struct Block {
    label: Option<String>,
    pub content: BlockContent,
    pub defs: Option<Vec<Definition>>,
}

impl Block {
    fn new(label: Option<String>, content: BlockContent) -> Self {
        Block {
            label,
            content,
            defs: None,
        }
    }
    pub fn is_code(&self) -> bool {
        matches!(self.content, BlockContent::Code(_))
    }
    fn new_start(label: Option<String>) -> Self {
        Self::new(label, BlockContent::Start)
    }
    fn new_stop(label: Option<String>) -> Self {
        Self::new(label, BlockContent::Stop)
    }
    fn new_code(label: Option<String>, code: Vec<Quadrupel>) -> Self {
        Self::new(label, BlockContent::Code(code))
    }

    fn contains_goto(&self) -> Option<String> {
        match &self.content {
            BlockContent::Code(quadrupels)
                if quadrupels
                    .last()
                    .is_some_and(|l| matches!(l.op, QuadrupelOp::Goto)) =>
            {
                Some(quadrupels.last().unwrap().result.to_string())
            }
            _ => None,
        }
    }

    fn contains_if(&self) -> Option<String> {
        match &self.content {
            BlockContent::Code(quadrupels)
                if quadrupels.last().is_some_and(|l| l.op.is_relop()) =>
            {
                Some(quadrupels.last().unwrap().result.to_string())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockContent {
    Start,
    Stop,
    Code(Vec<Quadrupel>),
}

#[derive(Debug, Default)]
pub struct BlockGraph {
    pub blocks: Vec<Block>,
    edges: Vec<HashSet<BlockId>>,
    label_to_id: HashMap<String, BlockId>,
}

impl BlockGraph {
    pub fn from_tac(code: &[Quadrupel]) -> Self {
        phase_3(phase_2(phase_1(code), code))
    }

    fn add_block(&mut self, block: Block, parent: Option<usize>) -> usize {
        if let Some(l) = block.clone().label {
            self.label_to_id.insert(l, self.blocks.len());
        }
        self.blocks.push(block);
        self.edges.push(HashSet::new());
        if let Some(parent) = parent {
            self.add_edge(parent, self.blocks.len() - 1);
        }

        self.blocks.len() - 1
    }

    fn add_edge(&mut self, start: usize, end: usize) {
        self.edges[start].insert(end);
    }

    pub fn edges(&self) -> &[HashSet<usize>] {
        &self.edges
    }

    pub fn path_exists(
        &self,
        start_block: usize,
        end_block: usize,
        def: &Definition,
        quad_nr: usize,
    ) -> bool {
        //println!("def: {:?}", def);
        //println!("path_exists: {} {}", start_block, end_block);
        let mut visited = vec![false; self.blocks.len()];
        let mut stack = vec![start_block];

        while let Some(current) = stack.pop() {
            let block = &self.blocks[current];
            if current == end_block {
                //println!("Found path to block {}", end_block);
                if block.is_code() {
                    let defs = block.defs.clone().unwrap();
                    if let Some(n) = defs.iter().find(|d| d.var == def.var) {
                        return quad_nr < n.quad_id;
                    }
                }
                return true;
            }

            if visited[current]
                || (current != start_block
                    && block.is_code()
                    && block.defs.clone().unwrap().iter().any(|d| d.var == def.var))
            {
                visited[current] = true;
                continue;
            }
            visited[current] = true;

            for &neighbor in &self.edges[current] {
                stack.push(neighbor);
            }
        }
        //println!("No path to block {}", end_block);
        false
    }

    fn new() -> Self {
        BlockGraph {
            blocks: vec![],
            edges: vec![],
            label_to_id: HashMap::new(),
        }
    }
}
