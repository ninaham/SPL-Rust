use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{QuadrupelArg, QuadrupelResult, QuadrupelVar},
    optimizations::{reaching_expressions::ReachingDefinitions, tarjan::Scc, worklist::Worklist},
    table::symbol_table::SymbolTable,
};

impl BlockGraph {
    pub fn loop_optimization(&mut self, local_table: &SymbolTable) -> bool {
        let mut repeat = false;
        self.tarjan();
        let mut sccs = self.sccs.take().unwrap();

        let mut block_counter = 0;

        for sccid in (0..sccs.len()).rev() {
            let revers_edges = self.edges_prev();
            let reachdef = ReachingDefinitions::run(self, local_table);

            let mut block_content = vec![];
            for &id in &sccs[sccid].nodes {
                let block = self
                    .blocks
                    .get_mut(id)
                    .unwrap_or_else(|| panic!("the world is mean to me"));
                match &mut block.content {
                    BlockContent::Code(quads) => {
                        for quad in quads.iter() {
                            if is_var_invariant(extract_var(&quad.arg1), &sccs[sccid], &reachdef)
                                && is_var_invariant(
                                    extract_var(&quad.arg2),
                                    &sccs[sccid],
                                    &reachdef,
                                )
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
                repeat = true;
                let parent_edges = revers_edges.get(sccs[sccid].nodes[0]).unwrap();
                let new_block = Block {
                    label: Some(format!("n{block_counter}")),
                    content: BlockContent::Code(block_content.clone()),
                };
                block_counter += 1;
                let new_id = self.add_block(new_block, None);
                for &parent in parent_edges {
                    if !sccs[sccid].nodes.contains(&parent) {
                        self.remove_edge(parent, sccs[sccid].nodes[0]);
                        self.add_edge(parent, new_id.clone());
                    }
                }
                self.add_edge(new_id, sccs[sccid].nodes[0]);
                //
                if let Some(pidx) = sccs[sccid].parent_idx {
                    sccs[pidx].nodes.push(new_id);
                }
            }
        }
        _ = self.sccs.insert(sccs);
        repeat
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
