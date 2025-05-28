use bitvec::vec::BitVec;

use crate::code_gen::quadrupel::QuadrupelVar;

use super::reaching_expressions::Definition;

pub struct LiveVariables {
    pub defs: Vec<QuadrupelVar>,
    pub def: Vec<BitVec>,
    pub use_bits: Vec<BitVec>,
    pub livin: Vec<BitVec>,
    pub livout: Vec<BitVec>,
}
