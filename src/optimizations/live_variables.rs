use std::collections::{HashSet, VecDeque};

use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockContent, BlockGraph};
use crate::code_gen::quadrupel::{QuadrupelOp, QuadrupelResult, QuadrupelVar};
use crate::optimizations::worklist::Definition;
use crate::table::symbol_table::SymbolTable;

pub struct LiveVariables {
    pub defs: Vec<Definition>,
    pub def: Vec<BitVec>,
    pub use_bits: Vec<BitVec>,
    pub livin: Vec<BitVec>,
    pub livout: Vec<BitVec>,
}

impl Block {
    pub fn defs_in_block_2(
        block_id: usize,
        defs_in_proc: &[Definition],
        unique_defs: HashSet<QuadrupelVar>,
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
        match self.content.clone() {
            BlockContent::Start => vec![],
            BlockContent::Stop => vec![],
            BlockContent::Code(quadrupels) => quadrupels
                .into_iter()
                .enumerate()
                .filter(|(_, quad)| {
                    matches!(
                        quad.op,
                        QuadrupelOp::Add
                            | QuadrupelOp::Mul
                            | QuadrupelOp::Div
                            | QuadrupelOp::Sub
                            | QuadrupelOp::Assign
                    )
                })
                .map(|(i, q)| {
                    (
                        i,
                        match q.result.clone() {
                            QuadrupelResult::Var(v) => v,
                            _ => unreachable!(),
                        },
                    )
                })
                .collect(),
        }
    }

    fn get_liv_use(&self, unique_defs: &HashSet<QuadrupelVar>) -> BitVec {
        let assignment_in_block = self.assignments_in_block();

        let used_vars = match &self.content {
            BlockContent::Start => vec![],
            BlockContent::Stop => vec![],
            BlockContent::Code(quadrupels) => quadrupels
                .iter()
                .enumerate()
                .flat_map(|(i, q)| {
                    vec![
                        match q.arg1.clone() {
                            crate::code_gen::quadrupel::QuadrupelArg::Var(quadrupel_var) => {
                                Some((i, quadrupel_var))
                            }
                            _ => None,
                        },
                        match q.arg2.clone() {
                            crate::code_gen::quadrupel::QuadrupelArg::Var(quadrupel_var) => {
                                Some((i, quadrupel_var))
                            }
                            _ => None,
                        },
                    ]
                })
                .flatten()
                .filter(|(i, v)| {
                    let assignment = assignment_in_block.iter().find(|(_, va)| v == va);
                    assignment.is_none() || assignment.unwrap().0 > *i
                })
                .collect::<Vec<_>>(),
        };

        unique_defs
            .iter()
            .map(|k| used_vars.iter().any(|d| &d.1 == k))
            .collect::<BitVec>()
    }
}

impl BlockGraph {
    pub fn live_variables(&mut self, local_table: &SymbolTable) -> LiveVariables {
        let defs_in_proc = self.definitions(local_table);
        let unique_defs = defs_in_proc
            .iter()
            .map(|d| d.var.clone())
            .collect::<HashSet<_>>();

        let defs = unique_defs
            .iter()
            .map(|qv| Definition {
                block_id: 0,
                quad_id: 0,
                var: qv.clone(),
            })
            .collect::<Vec<_>>();

        let def = self
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| {
                Block::defs_in_block_2(block_id, &defs_in_proc, unique_defs.clone())
            })
            .collect::<Vec<_>>();

        let r#use = self
            .blocks
            .iter()
            .map(|b| b.get_liv_use(&unique_defs))
            .collect::<Vec<_>>();

        let edges_prev = self.edges_prev();
        let edges = self.edges();

        let mut out: Vec<BitVec> = vec![BitVec::repeat(false, defs.len()); self.blocks.len()];
        let mut r#in: Vec<BitVec> = vec![BitVec::repeat(false, defs.len()); self.blocks.len()];
        let mut changed = VecDeque::from_iter((0..self.blocks.len()).rev());

        while let Some(node) = changed.pop_front() {
            for &s in &edges[node] {
                r#out[node] |= &r#in[s];
            }

            let in_first_part = out[node]
                .iter()
                .by_vals()
                .enumerate()
                .map(|(i, b)| if b { !def[node][i] } else { false })
                .collect::<BitVec>();

            let in_old = std::mem::replace(&mut r#in[node], in_first_part | &r#use[node]);

            if r#in[node] != in_old {
                changed.extend(&edges_prev[node]);
            }
        }
        LiveVariables {
            defs,
            def,
            use_bits: r#use,
            livin: r#in,
            livout: out,
        }
    }
}
