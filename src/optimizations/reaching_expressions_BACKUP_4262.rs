use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write;

use bitvec::vec::BitVec;

use crate::code_gen::quadrupel::{QuadrupelArg, QuadrupelOp, quad, quad_match};
use crate::table::entry::{Entry, Parameter};
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

<<<<<<< HEAD
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

=======
>>>>>>> 82cf5d3 (fix use)
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

<<<<<<< HEAD
    fn get_liv_use(&self, unique_defs: &HashSet<QuadrupelVar>) -> BitVec {
=======
    fn get_liv_use(&self, defs_in_proc: &[Definition]) -> BitVec {
>>>>>>> 82cf5d3 (fix use)
        let assignment_in_block = self.assignments_in_block();

        let used_vars = match self.content.clone() {
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

<<<<<<< HEAD
        unique_defs
            .iter()
            .map(|k| used_vars.iter().any(|d| &d.1 == k))
=======
        defs_in_proc
            .iter()
            .map(|k| used_vars.iter().any(|d| d.1 == k.var))
>>>>>>> 82cf5d3 (fix use)
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

    pub fn definitions(
        &mut self,
        block_id: usize,
        quads: &[Quadrupel],
        symbol_table: &SymbolTable,
    ) -> Vec<Definition> {
<<<<<<< HEAD
        quads
=======
        self.defs
            .get_or_insert_with(|| {
                quads
                    .iter()
                    .enumerate()
                    .filter_map(move |(i, q)| match q {
                        quad_match!((_), _, _ => QuadrupelResult::Var(v)) => Some(Definition {
                            block_id,
                            quad_id: i,
                            var: v.clone(),
                        }),
                        quad_match!((p), (~v), _ => _) => {
                            let param = Quadrupel::find_param_declaration(quads, i, symbol_table);

                            param.is_reference.then(|| Definition {
                                block_id,
                                quad_id: i,
                                var: v.clone(),
                            })
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .clone()
    }*/

    pub fn definitions(&mut self, block_id: usize, quads: &[Quadrupel]) -> Vec<Definition> {
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
            .collect::<Vec<_>>()
    }

    pub fn unique_definitions(&mut self, block_id: usize, quads: &[Quadrupel]) -> Vec<Definition> {
        let defs = quads
>>>>>>> 82cf5d3 (fix use)
            .iter()
            .enumerate()
            .filter_map(move |(i, q)| match q {
                quad_match!((_), _, _ => QuadrupelResult::Var(v)) => Some(Definition {
                    block_id,
                    quad_id: i,
                    var: v.clone(),
                }),
                quad_match!((p), (~v), _ => _) => {
                    let param = Quadrupel::find_param_declaration(quads, i, symbol_table);

                    param.is_reference.then(|| Definition {
                        block_id,
                        quad_id: i,
                        var: v.clone(),
                    })
                }
                _ => None,
            })
<<<<<<< HEAD
            .collect::<Vec<_>>()
=======
            .map(|d| (d.clone().var, d))
            .collect::<HashMap<_, _>>();
        defs.values().cloned().collect()
>>>>>>> 82cf5d3 (fix use)
    }
}

impl Quadrupel {
    pub fn find_param_declaration(
        quads: &[Quadrupel],
        quad_index_param: usize,
        symbol_table: &SymbolTable,
    ) -> Parameter {
        let (n, call) = quads
            .iter()
            .skip(quad_index_param)
            .filter(|q| q.op == QuadrupelOp::Param || q.op == QuadrupelOp::Call)
            .enumerate()
            .find(|(_, qc)| qc.op == QuadrupelOp::Call)
            .unwrap();
        let QuadrupelArg::Var(QuadrupelVar::Spl(ref call_name)) = call.arg1 else {
            unreachable!()
        };
        let Entry::ProcedureEntry(call_proc) = symbol_table.lookup(call_name).unwrap() else {
            unreachable!()
        };
        call_proc.parameters.into_iter().rev().nth(n - 1).unwrap()
    }
}

impl BlockGraph {
    pub fn live_variables(&mut self, local_table: &SymbolTable) -> LiveVariables {
<<<<<<< HEAD
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
=======
        let defs_in_proc = self.unique_definitions(local_table);
>>>>>>> 82cf5d3 (fix use)

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
<<<<<<< HEAD
            .map(|b| b.get_liv_use(&unique_defs))
=======
            .map(|b| b.get_liv_use(&defs_in_proc))
>>>>>>> 82cf5d3 (fix use)
            .collect::<Vec<_>>();

        let edges = self.edges();

        let mut out: Vec<BitVec> = vec![BitVec::repeat(false, defs.len()); self.blocks.len()];
        let mut r#in: Vec<BitVec> = vec![BitVec::repeat(false, defs.len()); self.blocks.len()];
        let mut changed = VecDeque::from_iter(0..self.blocks.len());

        while let Some(node) = changed.pop_back() {
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
            defs,
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
                    BlockContent::Code(quads) => self.blocks[i].definitions(i, quads, local_table),
                    BlockContent::Stop => vec![],
                }
            })
            .collect()
    }

    fn unique_definitions<'a>(&'a mut self, local_table: &'a SymbolTable) -> Vec<Definition> {
        let mut defs = HashMap::new();
        (0..self.blocks.len())
            .flat_map(|i| -> Vec<_> {
                match &self.blocks[i].clone().content {
                    BlockContent::Start => local_table.entries.iter().map(Into::into).collect(),
                    BlockContent::Code(quads) => self.blocks[i].unique_definitions(i, quads),
                    BlockContent::Stop => vec![],
                }
            })
            .for_each(|d| {
                if !defs.contains_key(&d.var) {
                    defs.insert(d.clone().var, d);
                }
            });

        defs.values().cloned().collect()
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
