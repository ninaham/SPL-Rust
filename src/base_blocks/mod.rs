use std::collections::{HashMap, HashSet};

use phaser::{phase_1, phase_2, phase_3};

use crate::code_gen::quadrupel::{Quadrupel, QuadrupelOp};

mod block_start_iter;
mod dot_graph;
mod phaser;
mod utils;

type BlockId = usize;

#[derive(Debug, Clone)]
pub struct Block {
    pub label: Option<String>,
    pub content: BlockContent,
}

impl Block {
    fn new(label: Option<String>, content: BlockContent) -> Self {
        Block { label, content }
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

#[derive(Debug, Default, Clone)]
pub struct BlockGraph {
    pub blocks: Vec<Block>,
    pub edges: Vec<HashSet<BlockId>>,
    pub label_to_id: HashMap<String, BlockId>,
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

    fn new() -> Self {
        BlockGraph {
            blocks: vec![],
            edges: vec![],
            label_to_id: HashMap::new(),
        }
    }
}
