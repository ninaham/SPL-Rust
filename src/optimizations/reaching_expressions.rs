use std::fmt::Write;
use std::{
    collections::{HashSet, VecDeque},
    iter,
};

use bitvec::vec::BitVec;

use crate::table::entry::Entry;
use crate::table::symbol_table::SymbolTable;
use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelResult, QuadrupelVar},
};

impl Block {
    fn get_rhc_gen(block_id: usize, defs_in_proc: &[Definition]) -> BitVec {
        defs_in_proc
            .iter()
            .map(|d| d.block_id == block_id)
            .collect::<BitVec>()
    }

    fn get_rch_prsrv(r#gen: &BitVec, defs_in_proc: &[Definition]) -> BitVec {
        let gen_vars = r#gen
            .iter()
            .enumerate()
            .filter(|(_, v)| *v.as_ref())
            .map(|(i, _)| &defs_in_proc[i].var)
            .collect::<Vec<_>>();

        defs_in_proc
            .iter()
            .map(|d| !gen_vars.contains(&&d.var))
            .collect::<BitVec>()
    }

    fn definitions(block_id: usize, quads: &[Quadrupel]) -> impl Iterator<Item = Definition> + '_ {
        quads
            .iter()
            .enumerate()
            .filter_map(move |(i, q)| match &q.result {
                QuadrupelResult::Var(v) => Some(Definition {
                    block_id,
                    quad_id: i,
                    var: v.clone(),
                }),
                _ => None,
            })
    }
}

impl BlockGraph {
    pub fn reaching_definitions(
        &self,
        local_table: &SymbolTable,
    ) -> (
        Vec<Definition>,
        Vec<BitVec>,
        Vec<BitVec>,
        Vec<BitVec>,
        Vec<BitVec>,
    ) {
        let defs_in_proc = self.definitions(local_table).collect::<Vec<_>>();

        let r#gen = self
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::get_rhc_gen(block_id, &defs_in_proc))
            .collect::<Vec<_>>();

        let prsv = r#gen
            .iter()
            .map(|r#gen| Block::get_rch_prsrv(r#gen, &defs_in_proc))
            .collect::<Vec<_>>();

        // worklist algorithm

        let edges_prev = self.edges_prev();
        let edges = self.edges();

        let mut out: Vec<BitVec> =
            vec![BitVec::repeat(false, defs_in_proc.len()); self.blocks.len()];
        let mut r#in: Vec<BitVec> =
            vec![BitVec::repeat(false, defs_in_proc.len()); self.blocks.len()];
        let mut changed = VecDeque::from_iter(0..self.blocks.len());

        while let Some(node) = changed.pop_front() {
            for &p in &edges_prev[node] {
                r#in[node] |= &out[p];
            }

            let out_old = std::mem::replace(
                &mut out[node],
                (r#in[node].clone() & &prsv[node]) | &r#gen[node],
            );

            if out[node] != out_old {
                changed.extend(&edges[node]);
            }
        }

        (defs_in_proc, r#gen, prsv, r#in, out)
    }

    fn definitions<'a>(
        &'a self,
        local_table: &'a SymbolTable,
    ) -> impl Iterator<Item = Definition> + 'a {
        self.blocks.iter().enumerate().flat_map(
            |(block_id, block)| -> Box<dyn Iterator<Item = Definition>> {
                match &block.content {
                    BlockContent::Start => Box::new(local_table.entries.iter().map(Into::into)),
                    BlockContent::Code(quads) => Box::new(Block::definitions(block_id, quads)),
                    BlockContent::Stop => Box::new(iter::empty()),
                }
            },
        )
    }

    fn edges_prev(&self) -> Vec<HashSet<usize>> {
        let mut edges_prev = vec![HashSet::new(); self.blocks.len()];

        self.edges()
            .iter()
            .enumerate()
            .for_each(|(block_id, edges)| {
                edges.iter().for_each(|&successor_id| {
                    edges_prev[successor_id].insert(block_id);
                });
            });

        edges_prev
    }
}

pub struct Definition {
    block_id: usize,
    quad_id: usize,
    var: QuadrupelVar,
}

impl From<(&String, &Entry)> for Definition {
    fn from((name, entry): (&String, &Entry)) -> Self {
        assert!(matches!(entry, Entry::VariableEntry(_)));
        Self {
            block_id: 0,
            quad_id: 0,
            var: QuadrupelVar::Spl(name.to_string()),
        }
    }
}

impl Definition {
    pub fn fmt_table<'a>(defs: impl Iterator<Item = &'a Self>) -> String {
        let mut out = String::new();

        writeln!(
            out,
            "{:>5}  {:<5} {:<5} {:<5}",
            "#", "Block", "Line", "Variable"
        )
        .unwrap();
        for (i, d) in defs.enumerate() {
            writeln!(out, "{i:>5}: {:>5} {:>5} {}", d.block_id, d.quad_id, d.var).unwrap();
        }

        out
    }
}
