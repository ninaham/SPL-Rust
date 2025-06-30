use crate::{base_blocks::BlockGraph, optimizations::live_variables::LiveVariables};

impl BlockGraph {
    pub fn loop_optimization(&mut self, live_variables: &LiveVariables) {
        if let Some(sccs_sorted) = self.sccs {
            self.sccs
                .as_mut()
                .expect("Here should be something")
                .sort_by_key(|scc| (scc.nodes.len()));

            for scc in sccs_sorted.iter() {
                for id in &scc.nodes {
                    let mut block = 0;
                }
            }
        };
    }
}
