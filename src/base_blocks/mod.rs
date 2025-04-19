use std::collections::HashMap;

use crate::code_gen::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult};

#[derive(Debug)]
pub struct Block {
    lable: Option<String>,
    content: BlockContent,
}
impl Block {
    fn new(lable: Option<String>, content: BlockContent) -> Self {
        Block { lable, content }
    }
    fn new_start(lable: Option<String>) -> Self {
        Self::new(lable, BlockContent::Start)
    }
    fn new_stop(lable: Option<String>) -> Self {
        Self::new(lable, BlockContent::Stop)
    }
    fn new_code(lable: Option<String>) -> Self {
        Self::new(lable, BlockContent::Code(Vec::new()))
    }

    fn get_code_mut(&mut self) -> &mut Vec<Quadrupel> {
        match self.content {
            BlockContent::Code(ref mut code) => code,
            _ => panic!("not a code block: {self:?}"),
        }
    }
}

#[derive(Debug)]
pub enum BlockContent {
    Start,
    Stop,
    Code(Vec<Quadrupel>),
}

#[derive(Default)]
pub struct BlockGraph {
    blocks: HashMap<usize, Block>,
    next_block_id: usize,
    edges: HashMap<usize, Vec<usize>>,
}

impl BlockGraph {
    pub fn from_tac(code: &[Quadrupel]) -> Self {
        let mut code = code.iter().cloned();
        let mut graph = BlockGraph::default();

        let block_start = Block::new_start(None);
        let (mut last_block_id, block_start) = graph.add_block(block_start, None);

        let mut block_active = Block::new_code(None);
        let (mut block_active_id, block_active) =
            graph.add_block(block_active, Some(last_block_id));
        let block_active_code = block_active.get_code_mut();

        while let Some(quad) = code.next() {
            match quad {
                Quadrupel {
                    op: QuadrupelOp::Default,
                    arg1: QuadrupelArg::Empty,
                    arg2: QuadrupelArg::Empty,
                    result: QuadrupelResult::Label(label),
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
        let id = self.next_block_id;
        self.next_block_id += 1;

        if let Some(parent) = parent {
            self.add_edge(parent, id);
        }

        let block = self.blocks.entry(id).insert_entry(block).into_mut();

        (id, block)
    }

    fn add_edge(&mut self, start: usize, end: usize) {
        self.edges.entry(start).or_default().push(end);
    }
}
