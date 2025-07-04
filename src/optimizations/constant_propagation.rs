use std::{collections::HashSet, fmt::Debug};

use colored::Colorize;

use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{
        Quadrupel, QuadrupelArg, QuadrupelResult, QuadrupelVar, quad, quad_match,
    },
    optimizations::worklist::{self, GetVarIdx, Lattice, LatticeJoinAssignCopy, Worklist},
    table::symbol_table::SymbolTable,
};

use self::Constness::{Constant, Undefined, Variable};

/// Enum representing the constantness of a variable:
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Constness {
    /// - Undefined: not yet determined
    Undefined,
    /// - Constant(i32): known constant value
    Constant(i32),
    /// - Variable: not a constant
    Variable,
}

// Custom debug formatting with colored output
impl Debug for Constness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Undefined => write!(f, "{}", "U".red()),
            Constant(c) => write!(f, "C{}", c.to_string().green()),
            Variable => write!(f, "{}", "V".blue()),
        }
    }
}

// Implementing the lattice operations needed for dataflow analysis
impl Lattice for Constness {
    fn init(_: usize) -> Self {
        Undefined
    }

    // Meet function used for merging information from predecessors
    fn meet(&self, other: &Self) -> Self {
        match (self, other) {
            (Undefined, c) | (c, Undefined) => *c,
            (Constant(v), Constant(w)) if v == w => Constant(*v),
            (Constant(_), Constant(_)) | (Variable, _) | (_, Variable) => Variable,
        }
    }

    // Join function used for merging information from successors
    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Variable, c) | (c, Variable) => *c,
            (Constant(v), Constant(w)) if v == w => Constant(*v),
            (Constant(_), Constant(_)) | (Undefined, _) | (_, Undefined) => Undefined,
        }
    }
}

// Enables use of default join+assign operations for this lattice
impl LatticeJoinAssignCopy for Constness {}

/// Struct representing the full state for the constant propagation analysis
pub struct ConstantPropagation {
    pub vars: Vec<QuadrupelVar>,   // All variables in the program
    pub gens: Vec<Vec<Constness>>, // GEN sets for each block
    pub prsv: Vec<Vec<Constness>>, // PRSV sets (what is preserved)
    pub r#in: Vec<Vec<Constness>>, // IN set per block
    pub out: Vec<Vec<Constness>>,  // OUT set per block
}

// Implement the worklist-based fixed-point algorithm
impl Worklist for ConstantPropagation {
    type Lattice = Vec<Constness>;
    type D = QuadrupelVar;

    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Forward;

    // Initialization of the lattice values and sets
    fn init(graph: &BlockGraph, local_table: &SymbolTable) -> Self {
        let vars = graph.all_vars(local_table);
        let gens = graph
            .blocks
            .iter()
            .map(|b| b.gcp_gen(&vars, local_table))
            .collect::<Vec<_>>();
        let prsv = gens.iter().map(|g| Block::gcp_prsv(g)).collect();

        Self {
            gens,
            prsv,
            r#in: Self::init_in_out(graph, &vars),
            out: Self::init_in_out(graph, &vars),
            vars,
        }
    }

    // Return mutable references to all state vectors
    fn state(&mut self) -> worklist::State<'_, Self> {
        worklist::State::<Self> {
            block_info_a: &mut self.gens,
            block_info_b: &mut self.prsv,
            input: &mut self.r#in,
            output: &mut self.out,
        }
    }
}

// Allows retrieving the index of a variable in the lattice vector
impl GetVarIdx<QuadrupelVar> for ConstantPropagation {
    fn vars(&self) -> &[QuadrupelVar] {
        &self.vars
    }
}

impl ConstantPropagation {
    /// Helper to get the constantness of a variable at a given program point
    pub fn get_constness(&self, var: &QuadrupelVar, const_state: &[Constness]) -> Constness {
        const_state[self.get_var_idx(var).unwrap()]
    }
}

// Extension method on the block graph to extract all defined variables
impl BlockGraph {
    fn all_vars(&self, local_table: &SymbolTable) -> Vec<QuadrupelVar> {
        self.definitions(local_table)
            .iter()
            .map(|d| d.var.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }
}

// Implement GEN and PRSV set computation for each block
impl Block {
    /// Compute the GEN set for constant propagation
    fn gcp_gen(&self, vars: &[QuadrupelVar], local_table: &SymbolTable) -> Vec<Constness> {
        let symbol_table = local_table.upper_level();
        let symbol_table = symbol_table.borrow();

        let var_idx = |var| ConstantPropagation::get_var_idx_in(vars, var).unwrap();

        match &self.content {
            // Start and Stop blocks produce nothing
            BlockContent::Start | BlockContent::Stop => vec![Undefined; vars.len()],
            BlockContent::Code(quads) => {
                let mut gens = vec![Undefined; vars.len()];

                for (i, quad) in quads.iter().enumerate() {
                    // Check if we're referencing a parameter
                    let is_reference =
                        || Quadrupel::find_param_declaration(quads, i, &symbol_table).is_reference;
                    Constness::from_quad(quad, &mut gens, var_idx, is_reference);
                }

                gens
            }
        }
    }

    /// Compute the PRSV set based on the GEN set
    fn gcp_prsv(gens: &[Constness]) -> Vec<Constness> {
        gens.iter()
            .map(|g| match g {
                Constant(_) | Variable => Undefined,
                Undefined => Variable,
            })
            .collect()
    }
}

impl Constness {
    /// Determine constness of a single argument (variable or constant)
    fn from_quad_arg<'a: 'b, 'b>(
        value: &'a QuadrupelArg,
        gens: &[Self],
        var_idx: impl Fn(&'b QuadrupelVar) -> usize,
    ) -> Self {
        match value {
            QuadrupelArg::Var(v) => match gens[var_idx(v)] {
                c @ Constant(_) => c,
                _ => Variable,
            },
            QuadrupelArg::Const(c) => Constant(*c),
            QuadrupelArg::Empty => Undefined,
        }
    }

    /// Update the GEN set based on the effect of a single Quadruple
    pub fn from_quad<'a: 'b, 'b>(
        quad: &'a Quadrupel,
        gens: &mut [Self],
        var_idx: impl Fn(&'b QuadrupelVar) -> usize,
        is_reference: impl FnOnce() -> bool,
    ) {
        match quad {
            // Handle assignment to a variable
            quad_match!(op, arg1, arg2 => res @ QuadrupelResult::Var(var)) => {
                let var = var_idx(var);
                let arg1 = Self::from_quad_arg(arg1, gens, &var_idx);
                let arg2 = Self::from_quad_arg(arg2, gens, &var_idx);

                gens[var] = match (op, (arg1, arg2)) {
                    // Simple constant assignment
                    (quad!(@op :=), (Constant(c), _)) => Constant(c),
                    // Unary minus
                    (quad!(@op ~ ), (Constant(c), _)) => Constant(-c),

                    // Binary operations with constant values
                    (op @ quad!(@op (+)(-)(*)(/)), (Constant(c1), Constant(c2))) => Constant(
                        quad!(*op, (=c1), (=c2) => res.clone())
                            .calc_const()
                            .unwrap(),
                    ),
                    // Otherwise, result is not constant
                    _ => Variable,
                };
            }
            // Special case: calling procedure by reference
            quad_match!((p), (~var), _ => _) if is_reference() => {
                let var = var_idx(var);
                // Cannot determine constness of references
                gens[var] = Variable;
            }
            _ => {}
        }
    }
}
