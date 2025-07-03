use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{QuadrupelArg, QuadrupelResult, QuadrupelVar},
    optimizations::{reaching_expressions::ReachingDefinitions, tarjan::Scc, worklist::Worklist},
    table::symbol_table::SymbolTable,
};

impl BlockGraph {
    pub fn loop_optimization(&mut self, local_table: &SymbolTable) {
        let revers_edges = self.edges_prev();
        let reachdef = ReachingDefinitions::run(self, local_table);
        if let Some(mut sccs_sorted) = self.sccs.clone() {
            sccs_sorted.sort_by_key(|scc| (scc.nodes.len()));

            let mut block_counter = 0;
            for scc in sccs_sorted.iter() {
                let mut block_content = vec![];
                for &id in &scc.nodes {
                    let block = self
                        .blocks
                        .get_mut(id)
                        .unwrap_or_else(|| panic!("the world is mean to me"));
                    match &mut block.content {
                        BlockContent::Code(quads) => {
                            for quad in quads.iter() {
                                if is_var_invariant(extract_var(&quad.arg1), &scc, &reachdef)
                                    && is_var_invariant(extract_var(&quad.arg2), &scc, &reachdef)
                                {
                                    match quad.result {
                                        QuadrupelResult::Var(_) => {
                                            block_content.push(quad.clone());
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            quads.retain_mut(|quad| !block_content.contains(&quad));
                        }
                        _ => {}
                    }
                }
                if !block_content.is_empty() {
                    let parent_edges = revers_edges.get(scc.nodes[0]).unwrap();
                    let new_block = Block {
                        label: Some(format!("n{block_counter}")),
                        content: BlockContent::Code(block_content.clone()),
                    };
                    block_counter += 1;
                    let new_id = self.add_block(new_block, None);
                    for &parent in parent_edges {
                        if !scc.nodes.contains(&parent) {
                            self.remove_edge(parent, scc.nodes[0]);
                            self.add_edge(parent, new_id.clone());
                        }
                    }
                    self.add_edge(new_id, scc.nodes[0]);
                }
            }
        };
    }
}

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
                if &def.var == var {
                    if blocks.nodes.contains(&def.block_id) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn extract_var(arg: &QuadrupelArg) -> Option<&QuadrupelVar> {
    if let QuadrupelArg::Var(var) = arg {
        Some(var)
    } else {
        None
    }
}
