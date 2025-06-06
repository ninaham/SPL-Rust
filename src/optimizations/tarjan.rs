use crate::base_blocks::{Block, BlockGraph};

#[derive(Debug, Default)]
pub struct SCC {
    scc: Vec<Vec<&Block>>,
}

impl BlockGraph {
    pub fn tarjan(&self) -> SCC {
        let mut stack: Vec<&Block>;

        SCC::default()
    }
}
