use std::{collections::HashSet, fmt::Debug};

use colored::Colorize;

use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{
        quad, quad_match, Quadrupel, QuadrupelArg, QuadrupelResult, QuadrupelVar,
    },
    optimizations::worklist::{self, GetVarIdx, Lattice, LatticeJoinAssignCopy, Worklist},
    table::symbol_table::SymbolTable,
};

use self::Constness::{Constant, Undefined, Variable};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Constness {
    Undefined,
    Constant(i32),
    Variable,
}

impl Debug for Constness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Undefined => write!(f, "{}", "U".red()),
            Constant(c) => write!(f, "C{}", c.to_string().green()),
            Variable => write!(f, "{}", "V".blue()),
        }
    }
}

impl Lattice for Constness {
    fn init(_: usize) -> Self {
        Undefined
    }

    fn meet(&self, other: &Self) -> Self {
        match (self, other) {
            (Undefined, c) | (c, Undefined) => *c,
            (Constant(v), Constant(w)) if v == w => Constant(*v),
            (Constant(_), Constant(_)) | (Variable, _) | (_, Variable) => Variable,
        }
    }

    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Variable, c) | (c, Variable) => *c,
            (Constant(v), Constant(w)) if v == w => Constant(*v),
            (Constant(_), Constant(_)) | (Undefined, _) | (_, Undefined) => Undefined,
        }
    }
}
impl LatticeJoinAssignCopy for Constness {}

pub struct ConstantPropagation {
    pub vars: Vec<QuadrupelVar>,
    pub gens: Vec<Vec<Constness>>,
    pub prsv: Vec<Vec<Constness>>,
    pub r#in: Vec<Vec<Constness>>,
    pub out: Vec<Vec<Constness>>,
}

impl Worklist for ConstantPropagation {
    type Lattice = Vec<Constness>;
    type D = QuadrupelVar;

    const EDGE_DIRECTION: worklist::EdgeDirection = worklist::EdgeDirection::Forward;

    fn init(graph: &mut BlockGraph, local_table: &SymbolTable) -> Self {
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

    fn state(&mut self) -> worklist::State<Self> {
        worklist::State::<Self> {
            block_info_a: &mut self.gens,
            block_info_b: &mut self.prsv,
            input: &mut self.r#in,
            output: &mut self.out,
        }
    }
}

impl GetVarIdx<QuadrupelVar> for ConstantPropagation {
    fn vars(&self) -> &[QuadrupelVar] {
        &self.vars
    }
}
impl ConstantPropagation {
    pub fn get_constness(&self, var: &QuadrupelVar, const_state: &[Constness]) -> Constness {
        const_state[self.get_var_idx(var).unwrap()]
    }
}

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

impl Block {
    fn gcp_gen(&self, vars: &[QuadrupelVar], local_table: &SymbolTable) -> Vec<Constness> {
        let symbol_table = local_table.upper_level();
        let symbol_table = symbol_table.lock().unwrap();

        let var_idx = |var| ConstantPropagation::get_var_idx_in(vars, var).unwrap();

        match &self.content {
            BlockContent::Start | BlockContent::Stop => vec![Undefined; vars.len()],
            BlockContent::Code(quads) => {
                let mut gens = vec![Undefined; vars.len()];

                for (i, quad) in quads.iter().enumerate() {
                    let is_reference =
                        || Quadrupel::find_param_declaration(quads, i, &symbol_table).is_reference;
                    Constness::from_quad(quad, &mut gens, var_idx, is_reference);
                }

                gens
            }
        }
    }

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

    pub fn from_quad<'a: 'b, 'b>(
        quad: &'a Quadrupel,
        gens: &mut [Self],
        var_idx: impl Fn(&'b QuadrupelVar) -> usize,
        is_reference: impl FnOnce() -> bool,
    ) {
        match quad {
            quad_match!(op, arg1, arg2 => res @ QuadrupelResult::Var(var)) => {
                let var = var_idx(var);
                let arg1 = Self::from_quad_arg(arg1, gens, &var_idx);
                let arg2 = Self::from_quad_arg(arg2, gens, &var_idx);

                gens[var] = match (op, (arg1, arg2)) {
                    (quad!(@op :=), (Constant(c), _)) => Constant(c),
                    (quad!(@op ~ ), (Constant(c), _)) => Constant(-c),

                    (op @ quad!(@op (+)(-)(*)(/)), (Constant(c1), Constant(c2))) => Constant(
                        quad!(*op, (=c1), (=c2) => res.clone())
                            .calc_const()
                            .unwrap(),
                    ),
                    _ => Variable,
                };
            }
            quad_match!((p), (~var), _ => _) if is_reference() => {
                let var = var_idx(var);
                // we dont know if procedure is const
                gens[var] = Variable;
            }
            _ => {}
        }
    }
}
