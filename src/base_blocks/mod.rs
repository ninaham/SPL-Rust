use std::{
    clone,
    collections::{HashMap, HashSet},
};

use crate::code_gen::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult};

mod phase2;
pub mod phase_3;

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
            BlockContent::Code(quadrupels) => match quadrupels.last().unwrap().op {
                QuadrupelOp::Equ
                | QuadrupelOp::Gre
                | QuadrupelOp::Grt
                | QuadrupelOp::Lse
                | QuadrupelOp::Lst
                | QuadrupelOp::Neq => Some(quadrupels.last().unwrap().result.to_string()),
                _ => None,
            },
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
        let code = code.iter().cloned();
        let mut graph = BlockGraph::default();

        let block_start = Block::new_start(None);
        let (last_block_id, block_start) = graph.add_block(block_start, None);

        let block_active = Block::new_code(None);
        let (block_active_id, block_active) = graph.add_block(block_active, Some(last_block_id));
        let block_active_code = block_active.get_code_mut();

        for quad in code {
            match quad {
                Quadrupel {
                    op: QuadrupelOp::Default,
                    arg1: QuadrupelArg::Empty,
                    arg2: QuadrupelArg::Empty,
                    result: QuadrupelResult::Label(_label),
                } => {
                    // TODO: new block
                }
                _ => block_active_code.push(quad),
            }
        }

        let block_stop = Block::new_stop(None);
        graph.add_block(block_stop, Some(block_active_id));

        graph
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
            _label_to_id: HashMap::new(),
        }
    }
}
