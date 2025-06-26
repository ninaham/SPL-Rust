#![expect(dead_code)]
use std::{cell::RefCell, collections::HashMap};

use crate::{
    code_gen::{
        Tac,
        quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar},
    },
    interpreter::{environment::Environment, value::Value},
};

pub fn eval_tac(tac: &Tac) {
    eval_function(tac, &"main".to_string(), &mut Vec::new());
}

pub fn eval_function(tac: &Tac, fun: &String, args: &mut Vec<Value>) {
    let instructions = find_function(tac, fun);
    let labels = label_indices(&instructions);
    let mut next_instruction = 0;
    let env = Environment {
        parent: None,
        vars: RefCell::new(HashMap::new()),
    };

    while next_instruction < instructions.len() {
        match eval_quad(tac, args, &instructions[next_instruction], &env, &labels) {
            Some(i) => next_instruction = i,
            None => next_instruction += 1,
        }
    }
}

pub fn find_function(tac: &Tac, fun: &String) -> Vec<Quadrupel> {
    tac.proc_table.get(fun).expect("function not found").clone()
}

pub fn label_indices(quads: &[Quadrupel]) -> HashMap<String, usize> {
    let mut labels = HashMap::new();

    for (i, quad) in quads.iter().enumerate() {
        if quad.op == QuadrupelOp::Default {
            match quad.result.clone() {
                QuadrupelResult::Var(_) => {}
                QuadrupelResult::Label(l) => {
                    labels.insert(l, i);
                }
                QuadrupelResult::Empty => unreachable!(),
            }
        }
    }

    labels
}

#[expect(clippy::too_many_lines)]
pub fn eval_quad(
    tac: &Tac,
    args: &mut Vec<Value>,
    quad: &Quadrupel,
    env: &Environment,
    labels: &HashMap<String, usize>,
) -> Option<usize> {
    match quad.op {
        QuadrupelOp::Add => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let res = parse_result(&quad.result);
            env.insert_val(&res, i + j);
            None
        }
        QuadrupelOp::Sub => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let res = parse_result(&quad.result);
            env.insert_val(&res, i - j);
            None
        }
        QuadrupelOp::Mul => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let res = parse_result(&quad.result);
            env.insert_val(&res, i * j);
            None
        }
        QuadrupelOp::Div => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let res = parse_result(&quad.result);
            env.insert_val(&res, i / j);
            None
        }
        QuadrupelOp::Neg => {
            let i = parse_arg(&quad.arg1, env);
            let res = parse_result(&quad.result);
            env.insert_val(&res, -i);
            None
        }
        QuadrupelOp::Equ => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let label = parse_result(&quad.result);

            if i == j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Neq => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let label = parse_result(&quad.result);

            if i == j {
                None
            } else {
                Some(*labels.get(&label).expect("no such label"))
            }
        }
        QuadrupelOp::Lst => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let label = parse_result(&quad.result);

            if i < j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Lse => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let label = parse_result(&quad.result);

            if i <= j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Grt => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let label = parse_result(&quad.result);

            if i > j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Gre => {
            let i = parse_arg(&quad.arg1, env);
            let j = parse_arg(&quad.arg2, env);
            let label = parse_result(&quad.result);

            if i >= j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Assign => {
            let val = parse_arg(&quad.arg1, env);
            let target = parse_result(&quad.result);

            env.insert_val(&target, val);
            None
        }
        QuadrupelOp::ArrayLoad => todo!(),
        QuadrupelOp::ArrayStore => todo!(),
        QuadrupelOp::Goto => {
            let label = parse_result(&quad.result);
            Some(*labels.get(&label).expect("no such label"))
        }
        QuadrupelOp::Param => {
            //let arg = parse_arg(&quad.arg1, env);
            //args.push(arg);
            None
        }
        QuadrupelOp::Call => {
            let fun = parse_fun(&quad.arg1);
            eval_function(tac, &fun, args);
            None
        }
        QuadrupelOp::Default => None,
    }
}

pub fn parse_arg<'a>(arg: &QuadrupelArg, env: &Environment<'a>) -> Value<'a> {
    match arg.clone() {
        QuadrupelArg::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => env.get(&name).expect("arg1 not found").borrow().clone(),
            QuadrupelVar::Tmp(t) => env
                .get(format!("T{t}").as_str())
                .expect("arg1 not found (temp)")
                .borrow()
                .clone(),
        },
        QuadrupelArg::Const(i) => Value::Int(i),
        QuadrupelArg::Empty => unreachable!(),
    }
}

pub fn parse_fun(arg: &QuadrupelArg) -> String {
    match arg.clone() {
        QuadrupelArg::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => name,
            QuadrupelVar::Tmp(t) => format!("T{t}"),
        },
        _ => unreachable!(),
    }
}

pub fn parse_result(res: &QuadrupelResult) -> String {
    match res.clone() {
        QuadrupelResult::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => name,
            QuadrupelVar::Tmp(t) => format!("T{t}"),
        },
        QuadrupelResult::Label(l) => l,
        QuadrupelResult::Empty => unreachable!(),
    }
}
