use crate::{
    base_blocks::BlockGraph,
    optimizations::{
        live_variables::LiveVariables, reaching_expressions::ReachingDefinitions,
        worklist::Worklist,
    },
    table::symbol_table::SymbolTable,
};

impl BlockGraph {
    pub fn loop_optimization(&mut self, local_table: &SymbolTable) {
        let reachdef = ReachingDefinitions::run(self, local_table);
        let live_variables = LiveVariables::run(self, local_table);
        if let Some(sccs_sorted) = self.sccs.clone() {
            self.sccs
                .as_mut()
                .expect("Here should be something")
                .sort_by_key(|scc| (scc.nodes.len()));

            for scc in sccs_sorted.iter() {
                for &id in &scc.nodes {
                    let mut block = self.blocks.get(id);
                    let liveout = live_variables.livout[id].clone();
                }
            }
        };
    }
}
