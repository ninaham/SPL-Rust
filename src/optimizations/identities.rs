use crate::code_gen::quadrupel::{Quadrupel, QuadrupelArg, quad, quad_match};

impl Quadrupel {
    /// Attempts to simplify a given quadrupel (quad) using constant folding and algebraic identities.
    pub fn simplify(self) -> Option<Self> {
        // Constant folding: if both arguments are constants, compute the result
        if let Some(val) = self.calc_const() {
            return Some(quad!((:=), (=val), _ => self.result));
        }

        // Constant comparison folding: if both arguments are constants, evaluate the condition
        if let Some(cond) = self.cmp_const() {
            return cond.then_some(quad!(=> self.result));
        }

        // Pattern-based algebraic simplifications
        Some(match self {
            // Addition/Subtraction with 0 or multiplication/division with 1
            quad_match!((+), 0, arg => res)
            | quad_match!((+)(-), arg, 0 => res)
            | quad_match!((*), 1, arg => res)
            | quad_match!((*)(/), arg, 1 => res) => {
                // a + 0 = a, a - 0 = a, a * 1 = a, a / 1 = a
                quad!((:=), (arg), _ => res)
            }

            // 0 - x = -x
            quad_match!((-), 0, arg => res) => {
                quad!((~), (arg), _ => res)
            }

            // x - x = 0
            quad_match!((-), arg1, arg2 => res) if arg1 == arg2 => {
                quad!((:=), 0, _ => res)
            }

            // x * 2 = x + x
            quad_match!((*), 2, arg => res) | quad_match!((*), arg, 2 => res) => {
                quad!((+), (arg.clone()), (arg) => res)
            }

            // x * 0 = 0, 0 * x = 0
            quad_match!((*)(/), 0, _ => res) | quad_match!((*), _, 0 => res) => {
                quad!((:=), 0, _ => res)
            }

            // No simplification pattern matched
            q => q,
        })
    }

    /// Performs constant folding on arithmetic operations if both operands are constants.
    pub const fn calc_const(&self) -> Option<i32> {
        let QuadrupelArg::Const(arg1) = self.arg1 else {
            return None;
        };

        // Second operand may be missing (e.g. unary operations)
        #[expect(clippy::match_wildcard_for_single_variants)]
        let arg2 = match self.arg2 {
            QuadrupelArg::Const(v) => Some(v),
            QuadrupelArg::Empty => None,
            _ => return None,
        };

        // Perform constant arithmetic depending on the operator
        match self.op {
            quad!(@op + ) => Some(arg1 + arg2.unwrap()),
            quad!(@op - ) => Some(arg1 - arg2.unwrap()),
            quad!(@op * ) => Some(arg1 * arg2.unwrap()),
            quad!(@op / ) => Some(arg1 / arg2.unwrap()),

            // Unary negation
            quad!(@op ~ ) => Some(-arg1),

            _ => None,
        }
    }

    /// Performs constant evaluation of comparison operations if both operands are constants.
    pub const fn cmp_const(&self) -> Option<bool> {
        let QuadrupelArg::Const(arg1) = self.arg1 else {
            return None;
        };

        let QuadrupelArg::Const(arg2) = self.arg2 else {
            return None;
        };

        // Perform constant comparison depending on the operator
        match self.op {
            quad!(@op == ) => Some(arg1 == arg2),
            quad!(@op != ) => Some(arg1 != arg2),
            quad!(@op <  ) => Some(arg1 < arg2),
            quad!(@op <= ) => Some(arg1 <= arg2),
            quad!(@op >  ) => Some(arg1 > arg2),
            quad!(@op >= ) => Some(arg1 >= arg2),

            _ => None,
        }
    }
}
