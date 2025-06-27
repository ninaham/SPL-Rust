use crate::{
    base_blocks::BlockGraph,
    optimizations::{
        constant_propagation::ConstantPropagation,
        live_variables::{self, LiveVariables},
        tarjan::Scc,
    },
};

impl BlockGraph {
    pub fn loop_optimization(
        &mut self,
        live_variables: &LiveVariables,
        constant: &ConstantPropagation,
    ) {
        let sccs: Vec<Scc> = self.sccs.unwrap_or(|| vec![]);
        sccs = sccs.sort_by_key(|scc| scc.nodes.len());

        for scc in sccs {}
    }
}
