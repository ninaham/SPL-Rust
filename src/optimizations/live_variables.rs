use std::collections::HashSet;

use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockContent, BlockGraph};
use crate::code_gen::quadrupel::{QuadrupelResult, QuadrupelVar};
use crate::optimizations::worklist::Definition;
use crate::table::symbol_table::SymbolTable;

use super::worklist::{self, Worklist};

pub struct LiveVariables {
    pub defs: Vec<Definition>,
    pub def: Vec<BitVec>,
    pub use_bits: Vec<BitVec>,
    pub livin: Vec<BitVec>,
    pub livout: Vec<BitVec>,
}

impl Worklist for LiveVariables {
    type Lattice = BitVec;
    type D = Definition;

    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Backward;

    fn init(
        state: &mut worklist::State<Self::Lattice, Self::D>,
        graph: &mut BlockGraph,
        local_table: &SymbolTable,
    ) {
        let defs_in_proc = graph.definitions(local_table);
        let unique_defs = defs_in_proc
            .iter()
            .map(|d| d.var.clone())
            .collect::<HashSet<_>>();

        let defs = unique_defs.iter().map(|qv| Definition {
            block_id: 0,
            quad_id: 0,
            var: qv.clone(),
        });

        let def = graph
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block_2(block_id, &defs_in_proc, &unique_defs));

        let r#use = graph.blocks.iter().map(|b| b.get_liv_use(&unique_defs));

        state.info_all.extend(defs);
        state.block_info_a.extend(r#use);
        state.block_info_b.extend(def);
    }

    fn output_first_part(
        state: &worklist::State<Self::Lattice, Self::D>,
        node: usize,
    ) -> Self::Lattice {
        state.input[node]
            .iter()
            .by_vals()
            .enumerate()
            .map(|(i, b)| b && !state.block_info_b[node][i])
            .collect::<BitVec>()
    }

    fn result(state: worklist::State<Self::Lattice, Self::D>) -> Self {
        Self {
            defs: state.info_all,
            def: state.block_info_b,
            use_bits: state.block_info_a,
            livin: state.output,
            livout: state.input,
        }
    }
}

impl Block {
    pub fn defs_in_block_2(
        block_id: usize,
        defs_in_proc: &[Definition],
        unique_defs: &HashSet<QuadrupelVar>,
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

    fn get_liv_use(&self, unique_defs: &HashSet<QuadrupelVar>) -> BitVec {
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
