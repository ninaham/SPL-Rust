use crate::code_gen::quadrupel::{quad, quad_match, Quadrupel, QuadrupelArg};

impl Quadrupel {
    #[expect(unused)]
    pub fn simplify(self) -> Option<Self> {
        if let Some(val) = self.calc_const() {
            return Some(quad!((:=), (=val), _ => self.result));
        }

        if let Some(cond) = self.cmp_const() {
            return match cond {
                true => Some(quad!(=> self.result)),
                false => None,
            };
        }

        Some(match self {
            quad_match!((+), 0, arg => res) | quad_match!((+)(-), arg, 0 => res) => {
                quad!((:=), (arg), _ => res)
            }
            quad_match!((-), 0, arg => res) => {
                quad!((~), (arg), _ => res)
            }

            quad_match!((*), 1, arg => res) | quad_match!((*)(/), arg, 1 => res) => {
                quad!((:=), (arg), _ => res)
            }

            quad_match!((*)(/), 0, _ => res) | quad_match!((*), _, 0 => res) => {
                quad!((:=), 0, _ => res)
            }

            q => q,
        })
    }

    pub const fn calc_const(&self) -> Option<i32> {
        let QuadrupelArg::Const(arg1) = self.arg1 else {
            return None;
        };

        let arg2 = match self.arg2 {
            QuadrupelArg::Const(v) => Some(v),
            QuadrupelArg::Empty => None,
            _ => return None,
        };

        match self.op {
            quad!(@op + ) => Some(arg1 + arg2.unwrap()),
            quad!(@op - ) => Some(arg1 - arg2.unwrap()),
            quad!(@op * ) => Some(arg1 * arg2.unwrap()),
            quad!(@op / ) => Some(arg1 / arg2.unwrap()),

            quad!(@op ~ ) => Some(-arg1),

            _ => None,
        }
    }

    pub const fn cmp_const(&self) -> Option<bool> {
        let QuadrupelArg::Const(arg1) = self.arg1 else {
            return None;
        };

        let QuadrupelArg::Const(arg2) = self.arg2 else {
            return None;
        };

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
