use crate::code_gen::quadrupel::QuadrupelOp;

impl QuadrupelOp {
    pub fn is_relop(&self) -> bool {
        matches!(
            self,
            Self::Equ | Self::Neq | Self::Lst | Self::Lse | Self::Grt | Self::Gre
        )
    }

    pub fn is_any_jump(&self) -> bool {
        matches!(self, Self::Goto) || self.is_relop()
    }
}
