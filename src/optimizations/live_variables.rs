use bitvec::vec::BitVec;

use super::reaching_expressions::Definition;

pub struct LiveVariables {
    pub defs: Vec<Definition>,
    pub def: Vec<BitVec>,
    pub use_bits: Vec<BitVec>,
    pub livin: Vec<BitVec>,
    pub livout: Vec<BitVec>,
}
