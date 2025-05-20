use bitvec::vec::BitVec;

use super::reaching_expressions::Definition;

pub struct LiveVariables {
    defs: Vec<Definition>,
    def: Vec<BitVec>,
    use_bits: Vec<BitVec>,
    livin: Vec<BitVec>,
    livout: Vec<BitVec>,
}
