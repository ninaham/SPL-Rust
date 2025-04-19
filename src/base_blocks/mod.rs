use std::collections::{HashMap, HashSet};

use phaser::{phase_1, phase_2, phase_3};

use crate::code_gen::quadrupel::{Quadrupel, QuadrupelOp};

mod block_start_iter;
mod phaser;
mod utils;

type BlockId = usize;

#[derive(Debug, Clone)]
pub struct Block {
    label: Option<String>,
    content: BlockContent,
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
    fn new_code(label: Option<String>) -> Self {
        Self::new(label, BlockContent::Code(Vec::new()))
    }

    fn get_code_mut(&mut self) -> &mut Vec<Quadrupel> {
        match self.content {
            BlockContent::Code(ref mut code) => code,
            _ => panic!("not a code block: {self:?}"),
        }
    }

    fn contains_goto(&self) -> Option<String> {
        match &self.content {
            BlockContent::Code(quadrupels) => match quadrupels.last().unwrap().op {
                QuadrupelOp::Goto => Some(quadrupels.last().unwrap().result.to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn contains_if(&self) -> Option<String> {
        match &self.content {
            BlockContent::Code(quadrupels) if quadrupels.last().unwrap().op.is_relop() => {
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

#[derive(Default)]
pub struct BlockGraph {
    pub blocks: Vec<Block>,
    edges: Vec<HashSet<BlockId>>,
    label_to_id: HashMap<String, BlockId>,
}

impl BlockGraph {
    pub fn from_tac(code: &[Quadrupel]) -> Self {
        phase_3(phase_2(phase_1(code), code))
    }

    fn add_block(&mut self, block: Block, parent: Option<usize>) -> (usize, &mut Block) {
        if let Some(l) = block.clone().label {
            self.label_to_id.insert(l, self.blocks.len());
        }
        self.blocks.push(block);
        if let Some(parent) = parent {
            self.add_edge(parent, self.blocks.len() - 1);
        }

        (self.blocks.len() - 1, self.blocks.last_mut().unwrap())
    }

    fn add_edge(&mut self, start: usize, end: usize) {
        self.edges[start].insert(end);
    }

    fn new() -> Self {
        BlockGraph {
            blocks: vec![],
            edges: vec![],
            label_to_id: HashMap::new(),
        }
    }
}
