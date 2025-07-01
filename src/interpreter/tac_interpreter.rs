use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    base_blocks::{BlockContent, BlockGraph},
    code_gen::quadrupel::{Quadrupel, QuadrupelArg, QuadrupelOp, QuadrupelResult, QuadrupelVar},
    interpreter::{
        environment::Environment,
        expression_evaluator,
        value::{Value, ValueFunction, ValueRef},
    },
    spl_builtins,
    table::{entry::Entry, symbol_table::SymbolTable},
};

pub fn eval_tac(
    proc_graphs: &HashMap<String, BlockGraph>,
    symbol_table: &Rc<RefCell<SymbolTable>>,
) {
    let procs = proc_graphs.iter().map(|(name, graph)| {
        let Some(Entry::ProcedureEntry(proc_entry)) = symbol_table.borrow().lookup(name) else {
            unreachable!("function not found: {name}");
        };
        (
            name.to_string(),
            Value::new_refcell(Value::Function(ValueFunction::Tac(proc_entry, graph))),
        )
    });
    let global_env = Rc::new(Environment::new_global(procs));

    spl_builtins::init_start_time();

    eval_function("main", &mut Vec::new(), global_env);
}

pub fn eval_function<'a>(fun: &str, args: &mut Vec<ValueRef<'a>>, parent_env: Rc<Environment<'a>>) {
    let proc = parent_env.get(fun).unwrap();
    let Value::Function(proc) = &*proc.borrow() else {
        unreachable!("function not found: {fun}");
    };

    let vars_param = args.drain(..).zip(proc.parameters()).map(|(arg, param)| {
        let param_name = param.name.to_string();
        if param.is_reference {
            (param_name, arg.clone())
        } else {
            (param_name, Value::new_refcell(arg.borrow().clone()))
        }
    });

    let vars_local = proc
        .local_table()
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
    let ValueFunction::Tac(_, proc_graph) = proc else {
        unreachable!();
    };

    let blocks = &proc_graph.blocks;
    let mut next_block = 0;

    while next_block < blocks.len() {
        match &blocks[next_block].content {
            BlockContent::Start => {
                next_block += 1;
            }
            BlockContent::Code(quads) => {
                for quad in quads {
                    if let Some(l) = &eval_quad(args, quad, env.clone()) {
                        next_block = *proc_graph.label_to_id.get(l).unwrap();
                        break;
                    }
                }
                next_block += 1;
            }
            BlockContent::Stop => {
                break;
            }
        }
    }
}

#[expect(clippy::too_many_lines)]
pub fn eval_quad<'a>(
    args: &mut Vec<ValueRef<'a>>,
    quad: &Quadrupel,
    env: Rc<Environment<'a>>,
) -> Option<String> {
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

            if i == j { Some(label) } else { None }
        }
        QuadrupelOp::Neq => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i == j { None } else { Some(label) }
        }
        QuadrupelOp::Lst => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i < j { Some(label) } else { None }
        }
        QuadrupelOp::Lse => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i <= j { Some(label) } else { None }
        }
        QuadrupelOp::Grt => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i > j { Some(label) } else { None }
        }
        QuadrupelOp::Gre => {
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            if i >= j { Some(label) } else { None }
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

            let index = eval_array_index(index, arr.len());
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
            let index = eval_array_index(index, arr.len());
            array[index] = Rc::new(RefCell::new(value));

            None
        }
        QuadrupelOp::Goto => {
            let label = parse_result(&quad.result);
            Some(label)
        }
        QuadrupelOp::Param => {
            let arg = parse_arg_ref(&quad.arg1, &env);
            args.push(arg);
            None
        }
        QuadrupelOp::Call => {
            let fun = parse_fun(&quad.arg1);
            eval_function(&fun, args, env);
            None
        }
        QuadrupelOp::Default => None,
    }
}

fn eval_array_index(index: i32, arr_len: usize) -> usize {
    expression_evaluator::eval_array_index(index / 4, arr_len)
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
    match &res {
        QuadrupelResult::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => name.to_string(),
            QuadrupelVar::Tmp(t) => format!("T{t}"),
        },
        QuadrupelResult::Label(l) => l.to_string(),
        QuadrupelResult::Empty => unreachable!(),
    }
}
