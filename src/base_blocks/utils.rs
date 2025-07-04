use crate::code_gen::quadrupel::QuadrupelOp;

impl QuadrupelOp {
    /// Checks if the operation is a relational operator (e.g., equality, inequality, less than, etc.).
    pub const fn is_relop(self) -> bool {
        matches!(
            self,
            Self::Equ | Self::Neq | Self::Lst | Self::Lse | Self::Grt | Self::Gre
        )
    }

    /// Checks if the operation is a jump operation (e.g., GOTO, or any relational operator).
    pub const fn is_any_jump(self) -> bool {
        matches!(self, Self::Goto) || self.is_relop()
    }
}
