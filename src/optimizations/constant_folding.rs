use std::ops::ControlFlow;

use bitvec::vec::BitVec;

use crate::{
    base_blocks::{BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp},
    optimizations::constant_propagation::{ConstantPropagation, Constness},
    optimizations::worklist::GetVarIdx,
    table::symbol_table::SymbolTable,
};

impl BlockGraph {
    /// Performs constant folding using current constant propagation state.
    /// Returns `ControlFlow::Break(())` if no changes were made (fixpoint reached),
    /// otherwise `ControlFlow::Continue(())` to indicate that further passes are needed.
    pub fn constant_folding(
        &mut self,
        const_prop: &mut ConstantPropagation,
        symbol_table: &SymbolTable,
    ) -> ControlFlow<()> {
        let mut stable = true;

        // Iterate over all basic blocks in the control flow graph
        for (block_id, block) in self.blocks.iter_mut().enumerate() {
            if let BlockContent::Code(quads) = &mut block.content {
                // Clone the current constant state for this block (input state)
                let mut const_state = const_prop.r#in[block_id].clone();

                // Detect which PARAM instructions refer to reference parameters
                let ref_params = quads
                    .iter()
                    .enumerate()
                    .map(|(i, q)| {
                        q.op == QuadrupelOp::Param
                            && Quadrupel::find_param_declaration(quads, i, symbol_table)
                                .is_reference
                    })
                    .collect::<BitVec>();

                // Iterate over each instruction in the block
                for (quad_idx, quad) in quads.iter_mut().enumerate() {
                    match &quad.op {
                        // Skip CALL instructions (side effects, no simplification)
                        QuadrupelOp::Call => {}

                        // Skip PARAM instructions that are reference arguments
                        QuadrupelOp::Param if ref_params[quad_idx] => {}

                        // All other instructions may be simplified
                        _ => {
                            // Replace variables with constants if known
                            const_prop.constant_folding(&mut quad.arg1, &const_state);
                            const_prop.constant_folding(&mut quad.arg2, &const_state);

                            // Attempt to simplify the instruction (e.g., fold constants)
                            *quad = quad.clone().simplify().unwrap_or(Quadrupel::EMPTY);

                            // Update constant propagation state with the result of the instruction
                            Constness::from_quad(
                                quad,
                                &mut const_state,
                                |var| const_prop.get_var_idx(var).unwrap(),
                                || ref_params[quad_idx],
                            );
                        }
                    }
                }

                // Remove empty instructions (e.g., those replaced by EMPTY)
                Quadrupel::filter_empty(quads);

                // Check if the output state has changed for this block
                if const_state != const_prop.out[block_id] {
                    stable = false;
                    const_prop.out[block_id] = const_state;
                }
            }
        }

        // Return whether the graph has reached a fixed point
        if stable {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl ConstantPropagation {
    /// Replaces a variable with a constant if the variable is known to be constant.
    fn constant_folding(&self, arg: &mut QuadrupelArg, const_state: &[Constness]) {
        if let QuadrupelArg::Var(arg_var) = &arg
            && let Constness::Constant(c) = self.get_constness(arg_var, const_state)
        {
            *arg = QuadrupelArg::Const(c);
        }
    }
}
