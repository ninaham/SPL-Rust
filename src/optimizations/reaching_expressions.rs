use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockGraph};
use crate::code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelVar};
use crate::optimizations::worklist::{self, Definition, Worklist};
use crate::table::entry::{Entry, Parameter};
use crate::table::symbol_table::SymbolTable;

pub struct ReachingDefinitions {
    pub defs: Vec<Definition>,
    pub gen_bits: Vec<BitVec>,
    pub prsv: Vec<BitVec>,
    pub rchin: Vec<BitVec>,
    pub rchout: Vec<BitVec>,
}

impl Worklist for ReachingDefinitions {
    type Lattice = BitVec;
    type D = Definition;

    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Forward;

    fn init(
        state: &mut worklist::State<Self::Lattice, Self::D>,
        graph: &mut BlockGraph,
        local_table: &SymbolTable,
    ) {
        let defs_all = graph.definitions(local_table);
        let defs_per_block = graph.defs_per_block(&defs_all);
        let r#gen = defs_per_block;
        let prsv = r#gen
            .iter()
            .map(|r#gen| Block::get_rch_prsrv(r#gen, &defs_all))
            .collect::<Vec<_>>();

        state.info_all = defs_all;
        state.block_info_a = r#gen;
        state.block_info_b = prsv;
    }

    fn result(state: worklist::State<Self::Lattice, Self::D>) -> Self {
        Self {
            defs: state.info_all,
            gen_bits: state.block_info_a,
            prsv: state.block_info_b,
            rchin: state.input,
            rchout: state.output,
        }
    }
}

impl Block {
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
}

impl Quadrupel {
    pub fn find_param_declaration(
        quads: &[Self],
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
