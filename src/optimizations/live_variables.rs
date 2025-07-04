use std::collections::HashSet;
use std::ops::Not;

use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockContent, BlockGraph};
use crate::code_gen::quadrupel::{QuadrupelResult, QuadrupelVar};
use crate::optimizations::worklist::Definition;
use crate::table::symbol_table::SymbolTable;

use super::worklist::{self, GetVarIdx, Worklist};

/// Struct representing the live variable analysis result.
pub struct LiveVariables {
    /// List of all variables in the procedure.
    pub vars: Vec<QuadrupelVar>,
    /// BitVec for each block indicating which variables are defined.
    pub def: Vec<BitVec>,
    /// BitVec for each block indicating which variables are used.
    pub use_bits: Vec<BitVec>,
    /// BitVec representing variables live at the entry of each block.
    pub livin: Vec<BitVec>,
    /// BitVec representing variables live at the exit of each block.
    pub livout: Vec<BitVec>,
}

impl Worklist for LiveVariables {
    type Lattice = BitVec;
    type D = QuadrupelVar;

    /// Live variable analysis flows backward.
    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Backward;

    /// Initializes the live variable analysis.
    fn init(graph: &BlockGraph, local_table: &SymbolTable) -> Self {
        // Get all variable definitions in the procedure.
        let defs_in_proc = graph.definitions(local_table);

        // Collect all unique variables that are defined.
        let vars = defs_in_proc
            .iter()
            .map(|d| d.var.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        // Compute 'def' bit vectors for each block.
        let def = graph
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block_2(block_id, &defs_in_proc, &vars))
            .collect();

        // Compute 'use' bit vectors for each block.
        let r#use = graph.blocks.iter().map(|b| b.get_liv_use(&vars)).collect();

        Self {
            def,
            use_bits: r#use,
            livin: Self::init_in_out(graph, &vars),
            livout: Self::init_in_out(graph, &vars),
            vars,
        }
    }

    /// Meet function for live variables:
    /// Intersection of current state and NOT of rhs (used in backward analysis).
    fn meet_override(lhs: &Self::Lattice, rhs: &Self::Lattice) -> Self::Lattice {
        rhs.clone().not() & lhs
    }

    /// Returns the current state needed by the generic worklist algorithm.
    fn state(&mut self) -> worklist::State<'_, Self> {
        worklist::State::<Self> {
            block_info_a: &mut self.use_bits, // use[B]
            block_info_b: &mut self.def,      // def[B]
            input: &mut self.livin,           // in[B]
            output: &mut self.livout,         // out[B]
        }
    }
}

/// Trait implementation to retrieve index of variables in the bitvectors.
impl GetVarIdx<QuadrupelVar> for LiveVariables {
    fn vars(&self) -> &[QuadrupelVar] {
        &self.vars
    }
}

impl Block {
    /// Computes a BitVec that indicates which variables from `unique_defs`
    /// are defined in the given block.
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

    /// Returns a list of assignments (index, variable) inside this block.
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

    /// Computes the use bit vector for live variable analysis.
    /// A variable is considered "used" if it is read before being (re)assigned.
    fn get_liv_use(&self, unique_defs: &[QuadrupelVar]) -> BitVec {
        let assignment_in_block = self.assignments_in_block();

        // Collect all variables that are used before they are assigned.
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
                    // Exclude uses that come after the variable is assigned.
                    let assignment = assignment_in_block.iter().find(|(_, va)| v == &va);
                    assignment.is_none() || assignment.unwrap().0 > *i
                })
                .collect::<Vec<_>>(),
        };

        // Mark the used variables in the BitVec.
        unique_defs
            .iter()
            .map(|k| used_vars.iter().any(|d| d.1 == k))
            .collect::<BitVec>()
    }
}
