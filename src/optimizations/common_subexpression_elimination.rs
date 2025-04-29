use super::aeb::AEBEntry;
use std::{collections::HashMap, rc::Rc};

use crate::{
    absyn::absyn::Expression,
    base_blocks::{Block, BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelOp, QuadrupelVar},
};

pub fn common_subexpression_elimination(graph: &mut BlockGraph) {
    graph.blocks.iter_mut().for_each(optimize_block);
}

pub fn optimize_block(block: &mut Block) {
    let aeb: Vec<AEBEntry> = Vec::new();
    let BlockContent::Code(quads) = &block.content else {
        return;
    };
    for (i, q) in quads.iter().enumerate() {
        match q.op {
            QuadrupelOp::Add
            | QuadrupelOp::Div
            | QuadrupelOp::Equ
            | QuadrupelOp::Gre
            | QuadrupelOp::Grt
            | QuadrupelOp::Lse
            | QuadrupelOp::Lst
            | QuadrupelOp::Mul
            | QuadrupelOp::Sub
            | QuadrupelOp::Neg
            | QuadrupelOp::Neq => todo!(),
            _ => continue,
        }
    }
}
