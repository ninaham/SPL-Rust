use super::aeb::AEBEntry;

use crate::{
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{
        Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar, quad,
    },
    table::{
        entry::{Entry, VariableEntry},
        symbol_table::SymbolTable,
        types::Type,
    },
};

impl BlockGraph {
    // Applies Common Subexpression Elimination (CSE) to all basic blocks in the graph.
    pub fn common_subexpression_elimination(&mut self, symbol_table: &mut SymbolTable) {
        // Find the highest temporary variable number used so far.
        let mut tmp_last_num = self
            .blocks
            .iter()
            .filter_map(|b| match &b.content {
                BlockContent::Code(quads) => Some(quads.iter().filter_map(|q| match q.result {
                    QuadrupelResult::Var(QuadrupelVar::Tmp(n)) => Some(n),
                    _ => None,
                })),
                _ => None,
            })
            .flatten()
            .max()
            .unwrap_or(0);

        // Function to generate new unique temporary variable numbers.
        let mut tmp_next_num = || -> usize {
            tmp_last_num += 10;
            tmp_last_num
        };

        // Apply the optimization to each block individually.
        self.blocks
            .iter_mut()
            .for_each(|b| optimize_block(b, &mut tmp_next_num, symbol_table));
    }
}

fn optimize_block(
    block: &mut Block,
    tmp_next_num: &mut impl FnMut() -> usize,
    local_table: &mut SymbolTable,
) {
    // Only process blocks that contain code.
    let BlockContent::Code(quads) = &mut block.content else {
        return;
    };

    let mut aeb: Vec<AEBEntry> = Vec::new(); // Available Expression Buffer
    let mut code_new = Vec::<Option<Quadrupel>>::new(); // New code vector

    // Iterate over each instruction in the block.
    for (i, quad) in quads.iter().cloned().enumerate() {
        // Try to simplify the instruction (e.g. constant folding).
        let Some(mut quad) = quad.simplify() else {
            continue;
        };

        let mut ref_param: Option<&dyn PartialEq<QuadrupelArg>> = None;

        match quad.op {
            // For binary or unary expressions, check if it has already been computed before.
            QuadrupelOp::Add
            | QuadrupelOp::Sub
            | QuadrupelOp::Mul
            | QuadrupelOp::Div
            | QuadrupelOp::Neg => {
                // Try to find a matching expression in the AEB (Available Expression Buffer).
                if let Some(entry) = aeb.iter_mut().find(|e| e.cmp(&quad)) {
                    // Reuse the result of the existing expression.
                    let tmp = entry.tmp.get_or_insert_with(|| {
                        let tmp = QuadrupelVar::Tmp(tmp_next_num());

                        local_table.enter(tmp.to_identifier(),
                            Entry::VariableEntry(VariableEntry
                            { typ: Type::INT, is_reference: false })).unwrap();

                        // Rewrite the original expression to assign to the temp variable.
                        let mut q = entry.quad.clone();
                        q.result = QuadrupelResult::Var(tmp.clone());
                        code_new[entry.pos - 1] = Some(q);

                        // Insert an assignment from the temp to the original result variable.
                        code_new[entry.pos] = Some(
                            quad!((:=), (QuadrupelArg::Var(tmp.clone())), _ => entry.quad.result.clone()),
                        );

                        tmp
                    });

                    // Replace current quad with a temp assignment instead.
                    quad = quad!((:=), (QuadrupelArg::Var(tmp.clone())), _ => quad.result);
                } else {
                    // No match found: keep original and store it in the AEB.
                    code_new.push(None);
                    aeb.push(AEBEntry::new(quad.clone(), code_new.len()));
                }
            }
            // Handle parameter passing for function calls.
            QuadrupelOp::Param => {
                let param = Quadrupel::find_param_declaration(quads, i, local_table);

                if param.is_reference {
                    // If passed by reference, remember the argument as potentially changed.
                    ref_param = Some(&quad.arg1);
                }

                // TODO: Eliminate repeated procedure calls as common subexpressions?
            }
            _ => {}
        }

        // Invalidate expressions in AEB that use the changed variable.
        let changed_var = ref_param.unwrap_or(&quad.result);
        aeb.retain(|e| changed_var != &e.quad.arg1 && changed_var != &e.quad.arg2);

        // Add the current instruction to the new code list.
        code_new.push(Some(quad));
    }

    // Replace original code with the optimized version (filtering out `None` entries).
    *quads = code_new.into_iter().flatten().collect();
}
