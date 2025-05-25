use std::collections::{HashSet, VecDeque};
use std::fmt::Write;

use bitvec::vec::BitVec;

use crate::table::entry::Entry;
use crate::table::symbol_table::SymbolTable;
use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelResult, QuadrupelVar},
};

use super::live_variables::LiveVariables;

pub struct ReachingDefinitions {
    pub defs: Vec<Definition>,
    pub gen_bits: Vec<BitVec>,
    pub prsv: Vec<BitVec>,
    pub rchin: Vec<BitVec>,
    pub rchout: Vec<BitVec>,
}

impl Block {
    pub fn defs_in_block(block_id: usize, defs_in_proc: &[Definition]) -> BitVec {
        defs_in_proc
            .iter()
            .map(|d| d.block_id == block_id)
            .collect::<BitVec>()
    }

    fn get_last_def<'a>(
        def_vars: &'a Vec<&Definition>,
        block_nr: usize,
        quad_nr: usize,
        var: &QuadrupelVar,
        graph: &BlockGraph,
    ) -> Vec<&'a &'a Definition> {
        let vars = def_vars
            .iter()
            .filter(|dv| {
                dv.var == var.clone() && graph.path_exists(dv.block_id, block_nr, dv, quad_nr)
            })
            .collect();
        vars
    }

    fn get_liv_use(
        def: &BitVec,
        defs_in_proc: &[Definition],
        block: &Block,
        block_nr: usize,
        graph: &BlockGraph,
    ) -> BitVec {
        let def_vars = def
            .iter()
            .by_vals()
            .enumerate()
            .map(|(i, _)| &defs_in_proc[i])
            .collect::<Vec<_>>();

        let used_vars = match block.content.clone() {
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
                .flat_map(|(i, var)| Self::get_last_def(&def_vars, block_nr, i, &var, graph))
                .collect::<Vec<_>>(),
        };

        def_vars
            .iter()
            .map(|k| used_vars.contains(&k))
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

    pub fn definitions(&mut self, block_id: usize, quads: &[Quadrupel]) -> Vec<Definition> {
        let defs = quads
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
            .collect::<Vec<_>>();
        self.defs = Some(defs.clone());
        defs
    }
}

impl BlockGraph {
    pub fn live_variables(&mut self, local_table: &SymbolTable) -> LiveVariables {
        let defs_in_proc = self.definitions(local_table);

        let def = self
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block(block_id, &defs_in_proc))
            .collect::<Vec<_>>();

        let r#use = self
            .blocks
            .iter()
            .enumerate()
            .map(|(i, b)| Block::get_liv_use(&def[i], &defs_in_proc, b, i, self))
            .collect::<Vec<_>>();

        //let edges_prev = self.edges_prev();
        let edges = self.edges();

        let mut out: Vec<BitVec> =
            vec![BitVec::repeat(false, defs_in_proc.len()); self.blocks.len()];
        let mut r#in: Vec<BitVec> =
            vec![BitVec::repeat(false, defs_in_proc.len()); self.blocks.len()];
        let mut changed = VecDeque::from_iter(0..self.blocks.len());

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
                changed.extend(&edges[node]);
            }
        }
        LiveVariables {
            defs: defs_in_proc,
            def,
            use_bits: r#use,
            livin: r#in,
            livout: out,
        }
    }

    pub fn reaching_definitions(&mut self, local_table: &SymbolTable) -> ReachingDefinitions {
        let defs_in_proc = self.definitions(local_table);

        let r#gen = self
            .blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block(block_id, &defs_in_proc))
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

        ReachingDefinitions {
            defs: defs_in_proc,
            gen_bits: r#gen,
            prsv,
            rchin: r#in,
            rchout: out,
        }
    }

    fn definitions<'a>(&'a mut self, local_table: &'a SymbolTable) -> Vec<Definition> {
        (0..self.blocks.len())
            .flat_map(|i| -> Vec<_> {
                match &self.blocks[i].clone().content {
                    BlockContent::Start => local_table.entries.iter().map(Into::into).collect(),
                    BlockContent::Code(quads) => self.blocks[i].definitions(i, quads),
                    BlockContent::Stop => vec![],
                }
            })
            .collect()
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Definition {
    block_id: usize,
    pub quad_id: usize,
    pub var: QuadrupelVar,
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
