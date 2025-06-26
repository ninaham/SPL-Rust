use std::collections::HashSet;
use std::ops::Not;

use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockContent, BlockGraph};
use crate::code_gen::quadrupel::{QuadrupelResult, QuadrupelVar};
use crate::optimizations::worklist::Definition;
use crate::table::symbol_table::SymbolTable;

use super::worklist::{self, GetVarIdx, Worklist};

pub struct LiveVariables {
    pub vars: Vec<QuadrupelVar>,
    pub def: Vec<BitVec>,
    pub use_bits: Vec<BitVec>,
    pub livin: Vec<BitVec>,
    pub livout: Vec<BitVec>,
}

impl Worklist for LiveVariables {
    type Lattice = BitVec;
    type D = QuadrupelVar;

    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Backward;

    fn init(graph: &mut BlockGraph, local_table: &SymbolTable) -> Self {
        let defs_in_proc = graph.definitions(local_table);
        let vars = defs_in_proc
            .iter()
            .map(|d| d.var.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let def = graph
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block_2(block_id, &defs_in_proc, &vars))
            .collect();

        let r#use = graph.blocks.iter().map(|b| b.get_liv_use(&vars)).collect();

        Self {
            def,
            use_bits: r#use,
            livin: Self::init_in_out(graph, &vars),
            livout: Self::init_in_out(graph, &vars),
            vars,
        }
    }

    fn meet_override(lhs: &Self::Lattice, rhs: &Self::Lattice) -> Self::Lattice {
        rhs.clone().not() & lhs
    }

    fn state(&mut self) -> worklist::State<Self> {
        worklist::State::<Self> {
            block_info_a: &mut self.use_bits,
            block_info_b: &mut self.def,
            input: &mut self.livin,
            output: &mut self.livout,
        }
    }
}

impl GetVarIdx<QuadrupelVar> for LiveVariables {
    fn vars(&self) -> &[QuadrupelVar] {
        &self.vars
    }
}

impl Block {
    pub fn defs_in_block_2(
        block_id: usize,
        defs_in_proc: &[Definition],
        unique_defs: &[QuadrupelVar],
    ) -> BitVec {
        let defs: Vec<_> = defs_in_proc
            .iter()
            .filter(|d| d.block_id == block_id)
            .cloned()
            .collect();

        unique_defs
            .iter()
            .map(|v| defs.iter().any(|d| d.var == *v))
            .collect::<BitVec>()
    }

    pub fn assignments_in_block(&self) -> Vec<(usize, QuadrupelVar)> {
        match self.content {
            BlockContent::Start | BlockContent::Stop => vec![],
            BlockContent::Code(ref quadrupels) => quadrupels
                .iter()
                .enumerate()
                .filter_map(|(i, q)| match &q.result {
                    QuadrupelResult::Var(v) => Some((i, v.clone())),
                    _ => None,
                })
                .collect(),
        }
    }

    fn get_liv_use(&self, unique_defs: &[QuadrupelVar]) -> BitVec {
        let assignment_in_block = self.assignments_in_block();

        let used_vars = match &self.content {
            BlockContent::Start | BlockContent::Stop => vec![],
            BlockContent::Code(quadrupels) => quadrupels
                .iter()
                .enumerate()
                .flat_map(|(i, q)| {
                    vec![
                        match &q.arg1 {
                            crate::code_gen::quadrupel::QuadrupelArg::Var(quadrupel_var) => {
                                Some((i, quadrupel_var))
                            }
                            _ => None,
                        },
                        match &q.arg2 {
                            crate::code_gen::quadrupel::QuadrupelArg::Var(quadrupel_var) => {
                                Some((i, quadrupel_var))
                            }
                            _ => None,
                        },
                    ]
                })
                .flatten()
                .filter(|(i, v)| {
                    let assignment = assignment_in_block.iter().find(|(_, va)| v == &va);
                    assignment.is_none() || assignment.unwrap().0 > *i
                })
                .collect::<Vec<_>>(),
        };

        unique_defs
            .iter()
            .map(|k| used_vars.iter().any(|d| d.1 == k))
            .collect::<BitVec>()
    }
}
