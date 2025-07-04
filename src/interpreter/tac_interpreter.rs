use std::{cell::RefCell, collections::HashMap, mem, rc::Rc};

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

// Evaluates the graphs of all procedure in a SPL program and initializes the global environment with procedures.
pub fn eval_tac(proc_graphs: &HashMap<String, BlockGraph>, symbol_table: &SymbolTable) {
    // Initialize the global environment with SPL procedures.
    let procs = proc_graphs.iter().map(|(name, graph)| {
        let Some(Entry::ProcedureEntry(proc_entry)) = symbol_table.lookup(name) else {
            unreachable!("function not found: {name}");
        };
        (
            name.to_string(),
            Value::new_refcell(Value::Function(ValueFunction::Tac(proc_entry, graph))),
        )
    });
    let global_env = Rc::new(Environment::new_global(procs, symbol_table));

    // get the program start for built-in time function
    spl_builtins::init_start_time();

    // Start the main procedure.
    eval_function("main", &mut Vec::new(), global_env);
}

pub fn eval_function<'a>(
    fun: &str,
    args: &mut Vec<ValueRef<'a>>,
    parent_env: Rc<Environment<'a, '_>>,
) {
    // Look up the function in the parent environment.
    let Some(Value::Function(proc)) = parent_env.get(fun).map(|v| v.borrow().clone()) else {
        unimplemented!("SPL-builtin `{fun}()`");
    };

    match proc {
        ValueFunction::Tac(_, proc_graph) => {
            // Create a new environment for the procedure call and initialize it with parameters and local variables.
            let vars_param = args
                .drain(..)
                .zip(&proc.entry().parameters)
                .map(|(arg, param)| {
                    let param_name = param.name.to_string();
                    if param.is_reference {
                        (param_name, arg.clone())
                    } else {
                        (param_name, Value::new_refcell(arg.borrow().clone()))
                    }
                });

            let vars_param_names = proc
                .entry()
                .parameters
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>();

            let vars_local = proc
                .entry()
                .local_table
                .entries
                .iter()
                .filter(|(n, _)| !vars_param_names.contains(n))
                .filter_map(|(name, entry)| {
                    let Entry::VariableEntry(var_entry) = entry else {
                        return None;
                    };
                    let default =
                        Rc::new(RefCell::new(var_entry.typ.default_value().flatten_value()));
                    Some((name.to_string(), default))
                });

            let env = Rc::new(Environment::new(
                parent_env,
                vars_param.chain(vars_local),
                &proc.entry().local_table,
            ));

            // Execute the procedure body in the new environment. Start with the first block.
            let blocks = &proc_graph.blocks;
            let mut next_block = 0;

            // Iterate through the blocks of the procedure graph.
            while next_block < blocks.len() {
                match &blocks[next_block].content {
                    BlockContent::Start => {
                        // If the block is a start block, continue to the next block.
                        next_block += 1;
                    }
                    BlockContent::Code(quads) => {
                        // If the block contains code, evaluate each quadrupel in the block.
                        for quad in quads {
                            // Evaluate the quadrupel and check if it returns a label for the next block.
                            if let Some(l) = &eval_quad(args, quad, env.clone()) {
                                next_block = *proc_graph.label_to_id.get(l).unwrap() - 1;
                            }
                        }
                        next_block += 1;
                    }
                    BlockContent::Stop => {
                        // If the block is a stop block, break out of the loop.
                        break;
                    }
                }
            }
        }
        ValueFunction::BuiltIn(_, f) => {
            // If the function is a built-in, call it with the provided arguments.
            f.call(args);
        }
        ValueFunction::Spl(_, _) => unreachable!(),
    }
}

// Evaluates a quadrupel and returns an optional label for the next block to execute.
#[expect(clippy::too_many_lines)]
pub fn eval_quad<'a>(
    args: &mut Vec<ValueRef<'a>>,
    quad: &Quadrupel,
    env: Rc<Environment<'a, '_>>,
) -> Option<String> {
    match quad.op {
        QuadrupelOp::Add => {
            // Evaluate operands and perform addition.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i + j;
            None
        }
        QuadrupelOp::Sub => {
            // Evaluate operands and perform subtraction.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i - j;
            None
        }
        QuadrupelOp::Mul => {
            // Evaluate operands and perform multiplication.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res)
                .unwrap_or_else(|| panic!("var not found: {res} {env:#?}"))
                .borrow_mut() = i * j;
            None
        }
        QuadrupelOp::Div => {
            // Evaluate operands and perform division.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = i / j;
            None
        }
        QuadrupelOp::Neg => {
            // Evaluate the argument and negate it.
            let i = parse_arg(&quad.arg1, &env);
            let res = parse_result(&quad.result);
            *env.get(&res).unwrap().borrow_mut() = -i;
            None
        }
        QuadrupelOp::Equ => {
            // Evaluate the arguments and check for equality.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            // If the values are equal, return the label; otherwise, return None.
            if i == j { Some(label) } else { None }
        }
        QuadrupelOp::Neq => {
            // Evaluate the arguments and check for inequality.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            // If the values are not equal, return the label; otherwise, return None.
            if i == j { None } else { Some(label) }
        }
        QuadrupelOp::Lst => {
            // Evaluate the arguments and check if the first is less than the second.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            // If the first value is less than the second, return the label; otherwise, return None.
            if i < j { Some(label) } else { None }
        }
        QuadrupelOp::Lse => {
            // Evaluate the arguments and check if the first is less than or equal to the second.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            // If the first value is less than or equal to the second, return the label; otherwise, return None.
            if i <= j { Some(label) } else { None }
        }
        QuadrupelOp::Grt => {
            // Evaluate the arguments and check if the first is greater than the second.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            // If the first value is greater than the second, return the label; otherwise, return None.
            if i > j { Some(label) } else { None }
        }
        QuadrupelOp::Gre => {
            // Evaluate the arguments and check if the first is greater than or equal to the second.
            let i = parse_arg(&quad.arg1, &env);
            let j = parse_arg(&quad.arg2, &env);
            let label = parse_result(&quad.result);

            // If the first value is greater than or equal to the second, return the label; otherwise, return None.
            if i >= j { Some(label) } else { None }
        }
        QuadrupelOp::Assign => {
            // Evaluate the argument and assign it to the result variable.
            let val = parse_arg(&quad.arg1, &env);
            let target = parse_result(&quad.result);

            // Update the environment with the new value.
            *env.get(&target).unwrap().borrow_mut() = val;
            None
        }
        QuadrupelOp::ArrayLoad => {
            // Load an element from an array at a specified index.
            // Parse the array and index arguments.
            let arr = parse_arg(&quad.arg1, &env);
            let Value::Array(arr) = arr else {
                unreachable!();
            };
            let Value::Int(index) = parse_arg(&quad.arg2, &env) else {
                unreachable!();
            };
            // Parse the result variable where the value will be stored.
            let res_id = parse_result(&quad.result);

            // Check if index is within bounds
            let index = eval_array_index(index, arr.len());

            // Get the result variable from the environment.
            let (res, is_ref) = env.get_mut(&res_id).unwrap();

            // If the result variable is a reference, we need to update it with the value from the array.
            if matches!(&*res.borrow(), Value::Int(_)) {
                // If the result is an integer, we clone the value from the array at the specified index.
                let ref_val = arr[index].clone();
                // If the result is a reference, we update the environment with the new value.
                // otherwise, we update the value in place.
                if is_ref {
                    mem::drop(res);
                    env.vars.borrow_mut().insert(res_id, ref_val);
                } else {
                    *res.borrow_mut() = ref_val.borrow().clone();
                }
            } else if matches!(&*res.borrow(), Value::Array(_)) {
                // If the result is an array, we create a new slice from the array starting at the specified index.
                let mut arr_slice = arr[index..].to_vec();
                // If the result is not a reference, we clone the values.
                if !is_ref {
                    for ele in &mut arr_slice {
                        let v = ele.borrow().clone();
                        *ele = Value::new_refcell(v);
                    }
                }
                // Update the result variable with the new array slice.
                *res.borrow_mut() = Value::Array(arr_slice);
            } else {
                unreachable!();
            }

            None
        }
        QuadrupelOp::ArrayStore => {
            // Store a value in an array at a specified index.
            // Parse the value to store, the index, and the array.
            let value = parse_arg(&quad.arg1, &env);
            let Value::Int(index) = parse_arg(&quad.arg2, &env) else {
                unreachable!();
            };
            let arr = parse_result(&quad.result);
            // Get the array from the environment.
            let array = env.get(&arr).expect("array not found");

            let &mut Value::Array(ref mut array) = &mut *array.borrow_mut() else {
                unreachable!();
            };

            // Check if index is within bounds
            let index = eval_array_index(index, array.len());
            // Store the value at the specified index in the array.
            *array[index].borrow_mut() = value;

            None
        }
        QuadrupelOp::Goto => {
            // Unconditional jump to a label.
            // Parse the label from the result and return it.
            let label = parse_result(&quad.result);
            Some(label)
        }
        QuadrupelOp::Param => {
            // Add a parameter to the argument list for a function call.
            // Parse the argument and add it to the list of arguments.
            let arg = parse_arg_ref(&quad.arg1, &env);
            args.push(arg);
            None
        }
        QuadrupelOp::Call => {
            // Call a function with the provided arguments.
            // Parse the function name and prepare to call it.
            let fun = parse_fun(&quad.arg1);
            eval_function(&fun, args, env);
            // Clear the arguments after the function call.
            args.clear();
            None
        }
        QuadrupelOp::Default => None,
    }
}

// Evaluates an array index, ensuring it is within bounds. All arrays are flat arrays of int values, so we divide the index by 4 to get the correct index in the array.
fn eval_array_index(index: i32, arr_len: usize) -> usize {
    expression_evaluator::eval_array_index(index / 4, arr_len)
}

// Parses a quadrupel argument and returns its value.
pub fn parse_arg<'a>(arg: &QuadrupelArg, env: &Rc<Environment<'a, '_>>) -> Value<'a> {
    parse_arg_ref(arg, env).borrow().clone()
}

// Parses a quadrupel argument and returns a reference to its value.
pub fn parse_arg_ref<'a>(arg: &QuadrupelArg, env: &Rc<Environment<'a, '_>>) -> ValueRef<'a> {
    match &arg {
        // If the argument is a variable, look it up in the environment.
        QuadrupelArg::Var(quadrupel_var) => env
            .get(&quadrupel_var.to_identifier())
            .unwrap_or_else(|| panic!("{arg:?} not found")),
        // If the argument is a temporary variable, create a new reference cell with the provided constant value.
        QuadrupelArg::Const(i) => Value::new_refcell(Value::Int(*i)),
        QuadrupelArg::Empty => unreachable!(),
    }
}

// Parses a quadrupel argument which refers to a function and returns its name.
pub fn parse_fun(arg: &QuadrupelArg) -> String {
    match arg.clone() {
        QuadrupelArg::Var(quadrupel_var) => match quadrupel_var {
            QuadrupelVar::Spl(name) => name,
            QuadrupelVar::Tmp(_) => unreachable!(),
        },
        _ => unreachable!(),
    }
}

// Parses a quadrupel result and returns its identifier as a string.
pub fn parse_result(res: &QuadrupelResult) -> String {
    match &res {
        QuadrupelResult::Var(quadrupel_var) => quadrupel_var.to_identifier(),
        QuadrupelResult::Label(l) => l.to_string(),
        QuadrupelResult::Empty => unreachable!(),
    }
}
