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
    pub fn constant_folding(
        &mut self,
        const_prop: &mut ConstantPropagation,
        symbol_table: &SymbolTable,
    ) -> ControlFlow<()> {
        let mut stable = true;

        for (block_id, block) in self.blocks.iter_mut().enumerate() {
            if let BlockContent::Code(quads) = &mut block.content {
                let mut const_state = const_prop.r#in[block_id].clone();
                let ref_params = quads
                    .iter()
                    .enumerate()
                    .map(|(i, q)| {
                        q.op == QuadrupelOp::Param
                            && Quadrupel::find_param_declaration(quads, i, symbol_table)
                                .is_reference
                    })
                    .collect::<BitVec>();

                for (quad_idx, quad) in quads.iter_mut().enumerate() {
                    match &quad.op {
                        QuadrupelOp::Call => {}
                        QuadrupelOp::Param if ref_params[quad_idx] => {}
                        _ => {
                            const_prop.constant_folding(&mut quad.arg1, &const_state);
                            const_prop.constant_folding(&mut quad.arg2, &const_state);
                            *quad = quad.clone().simplify().unwrap_or(Quadrupel::EMPTY);

                            Constness::from_quad(
                                quad,
                                &mut const_state,
                                |var| const_prop.get_var_idx(var).unwrap(),
                                || ref_params[quad_idx],
                            );
                        }
                    }
                }

                Quadrupel::filter_empty(quads);

                if const_state != const_prop.out[block_id] {
                    stable = false;
                    const_prop.out[block_id] = const_state;
                }
            }
        }

        if stable {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

impl ConstantPropagation {
    fn constant_folding(&self, arg: &mut QuadrupelArg, const_state: &[Constness]) {
        if let QuadrupelArg::Var(arg_var) = &arg
            && let Constness::Constant(c) = self.get_constness(arg_var, const_state)
        {
            *arg = QuadrupelArg::Const(c);
        }
    }
}
