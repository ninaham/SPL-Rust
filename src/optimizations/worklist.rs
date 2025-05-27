use std::collections::{HashSet, VecDeque};
use std::fmt::Write;
use std::iter;

use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockContent, BlockGraph};
use crate::code_gen::quadrupel::{quad, quad_match, Quadrupel, QuadrupelResult, QuadrupelVar};
use crate::table::entry::Entry;
use crate::table::symbol_table::SymbolTable;

pub struct State<L: Lattice, D> {
    pub info_all: Vec<D>,
    pub block_info_a: Vec<L>,
    pub block_info_b: Vec<L>,
    pub input: Vec<L>,
    pub output: Vec<L>,
}

pub trait Lattice: Clone + Eq {
    fn init(len: usize) -> Self;
    fn meet(&self, other: &Self) -> Self;
    fn join(&self, other: &Self) -> Self;
    fn join_assign(&mut self, other: &Self);
}

pub enum EdgeDirection {
    Forward,
    Backward,
}

pub trait Worklist {
    type Lattice: self::Lattice;
    type D;

    const EDGE_DIRECTION: self::EdgeDirection;

    fn init(
        state: &mut State<Self::Lattice, Self::D>,
        graph: &mut BlockGraph,
        local_table: &SymbolTable,
    );
    fn output_first_part(state: &State<Self::Lattice, Self::D>, node: usize) -> Self::Lattice {
        state.input[node].meet(&state.block_info_b[node])
    }
    fn result(state: State<Self::Lattice, Self::D>) -> Self;

    fn run(graph: &mut BlockGraph, local_table: &SymbolTable) -> Self
    where
        Self: Sized,
    {
        graph.run_worklist(local_table)
    }
}

impl BlockGraph {
    fn run_worklist<W: Worklist>(&mut self, local_table: &SymbolTable) -> W {
        let mut state = State::<W::Lattice, W::D> {
            info_all: Vec::new(),
            block_info_a: Vec::new(),
            block_info_b: Vec::new(),
            input: Vec::new(),
            output: Vec::new(),
        };

        W::init(&mut state, self, local_table);

        let mut edges_both = [self.edges(), &self.edges_prev()];
        if matches!(W::EDGE_DIRECTION, self::EdgeDirection::Backward) {
            edges_both.reverse();
        }
        let [edges_forward, edges_backward] = edges_both;

        state.output.extend(iter::repeat_n(
            W::Lattice::init(state.info_all.len()),
            self.blocks.len(),
        ));
        state.input.extend(iter::repeat_n(
            W::Lattice::init(state.info_all.len()),
            self.blocks.len(),
        ));
        let mut changed = match W::EDGE_DIRECTION {
            EdgeDirection::Forward => (0..self.blocks.len()).collect::<VecDeque<_>>(),
            EdgeDirection::Backward => (0..self.blocks.len()).rev().collect::<VecDeque<_>>(),
        };

        while let Some(node) = changed.pop_front() {
            for &p in &edges_backward[node] {
                state.input[node].join_assign(&state.output[p]);
            }

            let output_first_part = W::output_first_part(&state, node);

            let output_old = std::mem::replace(
                &mut state.output[node],
                output_first_part.join(&state.block_info_a[node]),
            );

            if state.output[node] != output_old {
                changed.extend(&edges_forward[node]);
            }
        }

        W::result(state)
    }

    pub fn definitions(&self, local_table: &SymbolTable) -> Vec<Definition> {
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

    pub fn edges_prev(&self) -> Vec<HashSet<usize>> {
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

    pub fn defs_per_block(&self, defs_in_proc: &[Definition]) -> Vec<BitVec> {
        self.blocks
            .iter()
            .enumerate()
            .map(|(block_id, _)| Block::defs_in_block(block_id, defs_in_proc))
            .collect::<Vec<_>>()
    }
}

impl Block {
    pub fn defs_in_block(block_id: usize, defs_in_proc: &[Definition]) -> BitVec {
        defs_in_proc
            .iter()
            .map(|d| d.block_id == block_id)
            .collect::<BitVec>()
    }

    pub fn definitions(
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

    fn join_assign(&mut self, other: &Self) {
        *self |= other;
    }
}
