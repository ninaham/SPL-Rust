use std::collections::{HashMap, HashSet};

use phaser::{phase_1, phase_2, phase_3};

use crate::{
    code_gen::quadrupel::{Quadrupel, QuadrupelOp},
    optimizations::tarjan::Scc,
};

mod block_start_iter;
mod dot_graph;
mod phaser;
mod utils;

pub type BlockId = usize;

/// A basic block, containing a label and content (start, stop, or code).
#[derive(Debug, Clone)]
pub struct Block {
    pub label: Option<String>,
    pub content: BlockContent,
}

impl Block {
    /// Creates a new block with given label and content.
    const fn new(label: Option<String>, content: BlockContent) -> Self {
        Self { label, content }
    }

    /// Creates a special 'start' block.
    const fn new_start(label: Option<String>) -> Self {
        Self::new(label, BlockContent::Start)
    }

    /// Creates a special 'stop' block.
    const fn new_stop(label: Option<String>) -> Self {
        Self::new(label, BlockContent::Stop)
    }

    /// Creates a block that contains code.
    const fn new_code(label: Option<String>, code: Vec<Quadrupel>) -> Self {
        Self::new(label, BlockContent::Code(code))
    }

    /// Checks if the block ends with a GOTO operation, and returns the target label if present.
    fn contains_goto(&self) -> Option<String> {
        match &self.content {
            BlockContent::Code(quads)
                if quads
                    .last()
                    .is_some_and(|l| matches!(l.op, QuadrupelOp::Goto)) =>
            {
                Some(quads.last().unwrap().result.to_string())
            }
            _ => None,
        }
    }

    /// Checks if the block ends with a conditional jump, and returns the target label if present.
    fn contains_if(&self) -> Option<String> {
        match &self.content {
            BlockContent::Code(quads) if quads.last().is_some_and(|l| l.op.is_relop()) => {
                Some(quads.last().unwrap().result.to_string())
            }
            _ => None,
        }
    }
}

/// Represents the content of a basic block.
#[derive(Debug, Clone)]
pub enum BlockContent {
    Start,
    Stop,
    Code(Vec<Quadrupel>),
}

/// Graph of basic blocks representing control flow.
#[derive(Debug, Default, Clone)]
pub struct BlockGraph {
    /// List of blocks in the graph.
    pub blocks: Vec<Block>,
    /// Edges representing control flow between blocks.
    pub edges: Vec<HashSet<BlockId>>,
    /// Mapping from block labels to their IDs.
    pub label_to_id: HashMap<String, BlockId>,
    /// Strongly connected components (optional).
    pub sccs: Option<Vec<Scc>>,
}

impl BlockGraph {
    /// Constructs a `BlockGraph` from a sequence of TAC instructions.
    pub fn from_tac(code: &[Quadrupel]) -> Self {
        phase_3(phase_2(phase_1(code), code))
    }

    /// Adds a new block to the graph, optionally linking it to a parent block.
    pub fn add_block(&mut self, block: Block, parent: Option<BlockId>) -> BlockId {
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

    /// Adds an edge between two blocks.
    pub fn add_edge(&mut self, start: BlockId, end: BlockId) {
        self.edges[start].insert(end);
    }

    /// Returns all edges of the graph.
    pub fn edges(&self) -> &[HashSet<usize>] {
        &self.edges
    }

    /// Removes an edge between two blocks.
    pub fn remove_edge(&mut self, start: BlockId, end: BlockId) {
        self.edges[start].remove(&end);
    }

    /// Creates an empty graph.
    fn new() -> Self {
        Self::default()
    }

    /// Gets the ID of a block by its label (panics if not found).
    #[expect(dead_code)]
    pub fn label_to_id(&self, label: &str) -> BlockId {
        *self.label_to_id.get(label).unwrap()
    }
}
