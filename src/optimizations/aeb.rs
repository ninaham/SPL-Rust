use crate::code_gen::quadrupel::{Quadrupel, QuadrupelVar};

pub(super) struct AEBEntry {
    quad: Quadrupel,
    pos: usize,
    tmp: Option<QuadrupelVar>,
}

impl PartialEq for AEBEntry {
    fn eq(&self, other: &Self) -> bool {
        false
    }
    fn ne(&self, other: &Self) -> bool {
        true
    }
}

impl Eq for AEBEntry {}
