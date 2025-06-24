use std::collections::{HashSet, VecDeque};
use std::fmt::Write;

use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockContent, BlockGraph};
use crate::cli::FmtTable;
use crate::code_gen::quadrupel::{Quadrupel, QuadrupelResult, QuadrupelVar, quad, quad_match};
use crate::table::entry::Entry;
use crate::table::symbol_table::SymbolTable;

pub struct State<'a, W: Worklist + ?Sized> {
    pub block_info_a: &'a mut [W::Lattice],
    pub block_info_b: &'a mut [W::Lattice],
    pub input: &'a mut [W::Lattice],
    pub output: &'a mut [W::Lattice],
}

pub trait Lattice: LatticeJoinAssign + Clone + Eq {
    fn init(len: usize) -> Self;
    fn meet(&self, other: &Self) -> Self;
    fn join(&self, other: &Self) -> Self;
}

pub trait LatticeJoinAssign {
    fn join_assign(&mut self, other: &Self);
}
pub trait LatticeJoinAssignCopy: Copy {}
impl<L: Lattice + LatticeJoinAssignCopy> LatticeJoinAssign for L {
    fn join_assign(&mut self, other: &Self) {
        *self = self.join(other);
    }
}

pub enum EdgeDirection {
    Forward,
    Backward,
}

pub trait Worklist {
    type Lattice: self::Lattice;
    type D;

    const EDGE_DIRECTION: self::EdgeDirection;

    fn init(graph: &mut BlockGraph, local_table: &SymbolTable) -> Self;
    fn meet_override(lhs: &Self::Lattice, rhs: &Self::Lattice) -> Self::Lattice {
        lhs.meet(rhs)
    }
    fn state(&mut self) -> State<'_, Self>;

    fn run(graph: &mut BlockGraph, local_table: &SymbolTable) -> Self
    where
        Self: Sized,
    {
        graph.run_worklist(local_table)
    }
    fn init_in_out(graph: &mut BlockGraph, info_all: &[Self::D]) -> Vec<Self::Lattice> {
        vec![Self::Lattice::init(info_all.len()); graph.blocks.len()]
    }
}

impl BlockGraph {
    fn run_worklist<W: Worklist>(&mut self, local_table: &SymbolTable) -> W {
        let mut state_res = W::init(self, local_table);
        let mut state = state_res.state();

        let mut edges_both = [self.edges(), &self.edges_prev()];
        if matches!(W::EDGE_DIRECTION, self::EdgeDirection::Backward) {
            edges_both.reverse();
        }
        let [edges_forward, edges_backward] = edges_both;

        if matches!(W::EDGE_DIRECTION, self::EdgeDirection::Backward) {
            std::mem::swap(&mut state.input, &mut state.output);
        }

        let mut changed = match W::EDGE_DIRECTION {
            EdgeDirection::Forward => (0..self.blocks.len()).collect::<VecDeque<_>>(),
            EdgeDirection::Backward => (0..self.blocks.len()).rev().collect::<VecDeque<_>>(),
        };

        while let Some(node) = changed.pop_front() {
            for &p in &edges_backward[node] {
                state.input[node].join_assign(&state.output[p]);
            }

            let output_first_part = W::meet_override(&state.input[node], &state.block_info_b[node]);

            let output_old = std::mem::replace(
                &mut state.output[node],
                output_first_part.join(&state.block_info_a[node]),
            );

            if state.output[node] != output_old {
                changed.extend(&edges_forward[node]);
            }
        }

        state_res
    }

    pub(super) fn definitions(&self, local_table: &SymbolTable) -> Vec<Definition> {
        (0..self.blocks.len())
            .flat_map(|i| -> Vec<_> {
                match &self.blocks[i].clone().content {
                    BlockContent::Start => local_table.entries.iter().map(Into::into).collect(),
                    BlockContent::Code(quads) => Block::definitions(i, quads, local_table),
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
                for &successor_id in edges {
                    edges_prev[successor_id].insert(block_id);
                }
            });

        edges_prev
    }

    pub(super) fn defs_per_block(&self, defs_in_proc: &[Definition]) -> Vec<BitVec> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block(block_id, defs_in_proc))
            .collect::<Vec<_>>()
    }
}

impl Block {
    fn defs_in_block(block_id: usize, defs_in_proc: &[Definition]) -> BitVec {
        defs_in_proc
            .iter()
            .map(|d| d.block_id == block_id)
            .collect::<BitVec>()
    }

    fn definitions(
        block_id: usize,
        quads: &[Quadrupel],
        symbol_table: &SymbolTable,
    ) -> Vec<Definition> {
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
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Definition {
    pub block_id: usize,
    pub quad_id: usize,
    pub var: QuadrupelVar,
}

impl FmtTable for Definition {
    fn fmt_table(defs: &[Self]) -> Result<String, std::fmt::Error> {
        let mut out = String::new();

        writeln!(
            out,
            "{:>5}  {:<5} {:<5} {:<5}",
            "#", "Block", "Line", "Variable"
        )?;
        for (i, d) in defs.iter().enumerate() {
            writeln!(out, "{i:>5}: {:>5} {:>5} {}", d.block_id, d.quad_id, d.var)?;
        }

        Ok(out)
    }
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

impl Lattice for BitVec {
    fn init(len: usize) -> Self {
        Self::repeat(false, len)
    }

    fn meet(&self, other: &Self) -> Self {
        self.clone() & other
    }

    fn join(&self, other: &Self) -> Self {
        self.clone() | other
    }
}

impl LatticeJoinAssign for BitVec {
    fn join_assign(&mut self, other: &Self) {
        *self |= other;
    }
}

impl<L: Lattice> Lattice for Vec<L> {
    fn init(len: usize) -> Self {
        vec![L::init(1); len]
    }

    fn meet(&self, other: &Self) -> Self {
        self.iter().zip(other).map(|(s, o)| L::meet(s, o)).collect()
    }

    fn join(&self, other: &Self) -> Self {
        self.iter().zip(other).map(|(s, o)| L::join(s, o)).collect()
    }
}

impl<L: Lattice> LatticeJoinAssign for Vec<L> {
    fn join_assign(&mut self, other: &Self) {
        for (s, o) in self.iter_mut().zip(other) {
            s.join_assign(o);
        }
    }
}

pub trait GetVarIdx<V: PartialEq> {
    fn vars(&self) -> &[V];

    fn get_var_idx_in(vars: &[V], var: &V) -> Option<usize> {
        vars.iter().position(|v| v == var)
    }

    fn get_var_idx(&self, var: &V) -> Option<usize> {
        Self::get_var_idx_in(self.vars(), var)
    }
}
