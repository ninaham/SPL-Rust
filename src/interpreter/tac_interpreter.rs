use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    code_gen::{
        quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar},
        Tac,
    },
    interpreter::{
        environment::Environment,
        expression_evaluator,
        value::{Value, ValueRef},
    },
    spl_builtins,
    table::entry::Entry,
};

pub fn eval_tac(tac: &Tac) {
    spl_builtins::init_start_time();
    eval_function(tac, &"main".to_string(), &mut Vec::new(), None);
}

pub fn eval_function<'a>(
    tac: &Tac,
    fun: &String,
    args: &mut Vec<ValueRef<'a>>,
    parent_env: Option<Rc<Environment<'a>>>,
) {
    let instructions = find_function(tac, fun);
    let labels = label_indices(&instructions);
    let mut next_instruction = 0;

    let Entry::ProcedureEntry(procedure_entry) = tac
        .symboltable
        .borrow()
        .lookup(fun)
        .expect("function not found")
    else {
        unreachable!();
    };

    let vars_param = args
        .drain(..)
        .zip(procedure_entry.parameters)
        .map(|(arg, param)| {
            if param.is_reference {
                (param.name, arg.clone())
            } else {
                (param.name, Value::new_refcell(arg.borrow().clone()))
            }
        });

    let vars_local = procedure_entry
        .local_table
        .entries
        .iter()
        .filter_map(|(name, entry)| {
            let Entry::VariableEntry(var_entry) = entry else {
                return None;
            };
            Some((
                name.to_string(),
                Value::new_refcell(var_entry.typ.default_value()),
            ))
        });

    let env = Rc::new(Environment::new(parent_env, vars_param.chain(vars_local)));

    while next_instruction < instructions.len() {
        match eval_quad(
            tac,
            args,
            &instructions[next_instruction],
            env.clone(),
            &labels,
        ) {
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
pub fn eval_quad<'a>(
    tac: &Tac,
    args: &mut Vec<ValueRef<'a>>,
    quad: &Quadrupel,
    env: Rc<Environment<'a>>,
    labels: &HashMap<String, usize>,
) -> Option<usize> {
    match quad.op {
        QuadrupelOp::Add => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i + j;
            None
        }
        QuadrupelOp::Sub => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i - j;
            None
        }
        QuadrupelOp::Mul => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i * j;
            None
        }
        QuadrupelOp::Div => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i / j;
            None
        }
        QuadrupelOp::Neg => {
            let i = parse_arg(&quad.arg1, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = -i;
            None
        }
        QuadrupelOp::Equ => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i == j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Neq => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i == j {
                None
            } else {
                Some(*labels.get(&label).expect("no such label"))
            }
        }
        QuadrupelOp::Lst => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i < j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Lse => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i <= j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Grt => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i > j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Gre => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i >= j {
                Some(*labels.get(&label).expect("no such label"))
            } else {
                None
            }
        }
        QuadrupelOp::Assign => {
            let val = parse_arg(&quad.arg1, &env);
            let target = parse_result(&quad.result);

            *env.get(&target).unwrap().borrow_mut() = val;
            None
        }
        QuadrupelOp::ArrayLoad => {
            let arr = parse_arg(&quad.arg1, &env);
            let Value::Array(arr) = arr else {
                unreachable!();
            };
            let Value::Int(index) = parse_arg(&quad.arg2, &env) else {
                unreachable!();
            };
            let res = parse_result(&quad.result);

            let index = eval_array_index(index, arr.len(), get_array_item_size(&arr));
            *env.get(&res).unwrap().borrow_mut() = arr[index].borrow().clone();
            None
        }
        QuadrupelOp::ArrayStore => {
            let value = parse_arg(&quad.arg1, &env);
            let Value::Int(index) = parse_arg(&quad.arg2, &env) else {
                unreachable!();
            };
            let arr = parse_result(&quad.result);
            let array = env.get(&arr).expect("array not found");

            let &mut Value::Array(ref mut array) = &mut *array.borrow_mut() else {
                unreachable!();
            };
            let index = eval_array_index(index, arr.len(), get_array_item_size(array));
            array[index] = Rc::new(RefCell::new(value));

            None
        }
        QuadrupelOp::Goto => {
            let label = parse_result(&quad.result);
            Some(*labels.get(&label).expect("no such label"))
        }
        QuadrupelOp::Param => {
            let arg = parse_arg_ref(&quad.arg1, &env);
            args.push(arg);
            None
        }
        QuadrupelOp::Call => {
            let fun = parse_fun(&quad.arg1);
            eval_function(tac, &fun, args, Some(env));
            None
        }
        QuadrupelOp::Default => None,
    }
}

fn eval_array_index(index: i32, arr_len: usize, index_factor: i32) -> usize {
    expression_evaluator::eval_array_index(index / index_factor, arr_len)
}

pub fn parse_arg<'a>(arg: &QuadrupelArg, env: &Rc<Environment<'a>>) -> Value<'a> {
    parse_arg_ref(arg, env).borrow().clone()
}
pub fn parse_arg_ref<'a>(arg: &QuadrupelArg, env: &Rc<Environment<'a>>) -> ValueRef<'a> {
    match arg.clone() {
        QuadrupelArg::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => env.get(&name).expect("arg1 not found"),
            QuadrupelVar::Tmp(t) => env
                .get(format!("T{t}").as_str())
                .expect("arg1 not found (temp)"),
        },
        QuadrupelArg::Const(i) => Value::new_refcell(Value::Int(i)),
        QuadrupelArg::Empty => unreachable!(),
    }
}

pub fn get_array_item_size(arr: &[Rc<RefCell<Value<'_>>>]) -> i32 {
    // TODO: Shouldn't this always be size_of(int)? (arrays are always flattened)
    if arr.is_empty() {
        panic!("Bitte komm einfach niemals vor")
    } else {
        get_size(&arr[0].borrow().clone())
    }
}

pub fn get_size(arr: &Value) -> i32 {
    match arr {
        Value::Array(arr) => {
            if arr.is_empty() {
                0
            } else {
                i32::try_from(arr.len()).unwrap() * get_size(&arr[0].borrow().clone())
            }
        }
        Value::Bool(_) | Value::Int(_) => 4,
        Value::Function(_) => unreachable!(),
    }
}

pub fn parse_fun(arg: &QuadrupelArg) -> String {
    match arg.clone() {
        QuadrupelArg::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => name,
            QuadrupelVar::Tmp(_) => unreachable!(),
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
