use bitvec::vec::BitVec;

use crate::base_blocks::{Block, BlockGraph};
use crate::code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelVar};
use crate::optimizations::worklist::{self, Definition, Worklist};
use crate::table::entry::{Entry, Parameter};
use crate::table::symbol_table::SymbolTable;

/// Struct representing the Reaching Definitions dataflow analysis.
pub struct ReachingDefinitions {
    /// All definitions in the program.
    pub defs: Vec<Definition>,
    /// GEN sets for each block (which definitions are generated).
    pub gen_bits: Vec<BitVec>,
    /// PRSV sets for each block (which definitions are preserved).
    pub prsv: Vec<BitVec>,
    /// Reaching definitions at block entry.
    pub rchin: Vec<BitVec>,
    /// Reaching definitions at block exit.
    pub rchout: Vec<BitVec>,
}

impl Worklist for ReachingDefinitions {
    type Lattice = BitVec;
    type D = Definition;

    /// Reaching definitions are a forward dataflow analysis.
    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Forward;

    /// Initializes the analysis: computes GEN and PRSV sets,
    /// and allocates empty IN and OUT sets.
    fn init(graph: &BlockGraph, local_table: &SymbolTable) -> Self {
        let defs_all = graph.definitions(local_table);
        let defs_per_block = graph.defs_per_block(&defs_all);
        let r#gen = defs_per_block;
        let prsv = r#gen
            .iter()
            .map(|r#gen| Block::get_rch_prsrv(r#gen, &defs_all))
            .collect::<Vec<_>>();

        Self {
            gen_bits: r#gen,
            prsv,
            rchin: Self::init_in_out(graph, &defs_all),
            rchout: Self::init_in_out(graph, &defs_all),
            defs: defs_all,
        }
    }

    /// Provides access to all analysis sets used by the worklist algorithm.
    fn state(&mut self) -> worklist::State<'_, Self> {
        worklist::State::<Self> {
            block_info_a: &mut self.gen_bits,
            block_info_b: &mut self.prsv,
            input: &mut self.rchin,
            output: &mut self.rchout,
        }
    }
}

impl Block {
    /// Computes the PRSV set: all definitions that are *not* killed by this block.
    /// That is, definitions of variables not redefined by the block.
    fn get_rch_prsrv(r#gen: &BitVec, defs_in_proc: &[Definition]) -> BitVec {
        // Extract the variables defined in this block (from GEN set).
        let gen_vars = r#gen
            .iter()
            .enumerate()
            .filter(|(_, v)| *v.as_ref())
            .map(|(i, _)| &defs_in_proc[i].var)
            .collect::<Vec<_>>();

        // Keep definitions whose variables are *not* redefined in this block.
        defs_in_proc
            .iter()
            .map(|d| !gen_vars.contains(&&d.var))
            .collect::<BitVec>()
    }
}

impl Quadrupel {
    /// Given the index of a `param` quad, finds the corresponding formal parameter
    /// declaration in the called procedure using the symbol table.
    pub fn find_param_declaration(
        quads: &[Self],
        quad_index_param: usize,
        symbol_table: &SymbolTable,
    ) -> Parameter {
        // Find the nth parameter by counting from the param to the corresponding call.
        let (n, call) = quads
            .iter()
            .skip(quad_index_param)
            .filter(|q| q.op == QuadrupelOp::Param || q.op == QuadrupelOp::Call)
            .enumerate()
            .find(|(_, qc)| qc.op == QuadrupelOp::Call)
            .unwrap();

        // Extract the procedure name from the call instruction.
        let QuadrupelArg::Var(QuadrupelVar::Spl(ref call_name)) = call.arg1 else {
            unreachable!()
        };

        // Lookup the procedure entry in the symbol table.
        let Entry::ProcedureEntry(call_proc) = symbol_table.lookup(call_name).unwrap() else {
            unreachable!()
        };

        // Return the nth parameter (counted in reverse).
        call_proc.parameters.into_iter().rev().nth(n - 1).unwrap()
    }
}
