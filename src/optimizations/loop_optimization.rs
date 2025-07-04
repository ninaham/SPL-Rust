use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{QuadrupelArg, QuadrupelResult, QuadrupelVar},
    optimizations::{reaching_expressions::ReachingDefinitions, tarjan::Scc, worklist::Worklist},
    table::symbol_table::SymbolTable,
};

impl BlockGraph {
    /// Performs loop-invariant code motion (LICM) optimization on the control flow graph.
    /// Returns `true` if any optimization was performed, otherwise `false`.
    pub fn loop_optimization(&mut self, local_table: &SymbolTable) -> bool {
        let mut repeat = false;

        // Identify strongly connected components (SCCs) using Tarjan's algorithm
        self.tarjan();
        let mut sccs = self.sccs.take().unwrap();

        let mut block_counter = 0;

        // Process SCCs in reverse order (from innermost loops outward)
        for sccid in (0..sccs.len()).rev() {
            let revers_edges = self.edges_prev(); // Get reverse edges for parent lookup
            let reachdef = ReachingDefinitions::run(self, local_table); // Run reaching definitions analysis

            let mut block_content = vec![];

            // Iterate over all basic blocks in the current SCC
            for &id in &sccs[sccid].nodes {
                let block = self
                    .blocks
                    .get_mut(id)
                    .unwrap_or_else(|| panic!("the world is mean to me"));

                // Only process blocks containing code (i.e., not entry/exit/other metadata blocks)
                if let BlockContent::Code(quads) = &mut block.content {
                    for quad in quads.iter() {
                        // Check if both operands are loop-invariant
                        if is_var_invariant(extract_var(&quad.arg1), &sccs[sccid], &reachdef)
                            && is_var_invariant(extract_var(&quad.arg2), &sccs[sccid], &reachdef)
                        {
                            // If the result is a variable, collect it for hoisting
                            if let QuadrupelResult::Var(_) = quad.result {
                                block_content.push(quad.clone());
                            }
                        }
                    }

                    // Remove the hoisted instructions from the original block
                    quads.retain_mut(|quad| !block_content.contains(quad));
                }
            }

            // If any loop-invariant code was found
            if !block_content.is_empty() {
                repeat = true;

                // Find predecessors of the loop header (entry into the SCC)
                let parent_edges = revers_edges.get(sccs[sccid].nodes[0]).unwrap();

                // Create a new block that holds the hoisted loop-invariant code
                let new_block = Block {
                    label: Some(format!("n{block_counter}")),
                    content: BlockContent::Code(block_content.clone()),
                };
                block_counter += 1;
                let new_id = self.add_block(new_block, None);

                // Redirect edges from parents outside the loop to the new block
                for &parent in parent_edges {
                    if !sccs[sccid].nodes.contains(&parent) {
                        self.remove_edge(parent, sccs[sccid].nodes[0]);
                        self.add_edge(parent, new_id);
                    }
                }

                // Connect the new block to the original loop header
                self.add_edge(new_id, sccs[sccid].nodes[0]);

                // Update SCC hierarchy if necessary
                if let Some(pidx) = sccs[sccid].parent_idx {
                    sccs[pidx].nodes.push(new_id);
                }
            }
        }

        // Store modified SCCs back in the graph
        _ = self.sccs.insert(sccs);
        repeat
    }
}

/// Checks whether a variable is loop-invariant with respect to a given SCC.
/// A variable is loop-invariant if none of its reaching definitions are inside the loop.
fn is_var_invariant(
    var: Option<&QuadrupelVar>,
    blocks: &Scc,
    reaching: &ReachingDefinitions,
) -> bool {
    if let Some(var) = var {
        let rchin = &reaching.rchin[blocks.nodes[0]];
        for (i, bit) in rchin.iter().enumerate() {
            if *bit {
                let def = &reaching.defs[i];
                if &def.var == var && blocks.nodes.contains(&def.block_id) {
                    return false; // Variable is defined inside the loop — not invariant
                }
            }
        }
    }
    true // No definitions inside the loop — invariant
}

/// Extracts a variable from a `QuadrupelArg`, if it is a variable.
const fn extract_var(arg: &QuadrupelArg) -> Option<&QuadrupelVar> {
    if let QuadrupelArg::Var(var) = arg {
        Some(var)
    } else {
        None
    }
}
