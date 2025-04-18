pub mod build_symbol_table;
pub mod table_initializer;
mod utils;

use std::fmt;

use crate::{
    absyn::{
        absyn::{Definition, Expression, Statement, Variable},
        procedure_definition::ProcedureDefinition,
    },
    table::{
        entry::Entry,
        symbol_table::SymbolTable,
        types::{PrimitiveType, Type},
    },
};

#[derive(Debug)]
pub struct SemanticError {
    pub _msg: String,
    //pos: i64,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SemanticError: \"{}\"", self._msg)?;
        //write!(f, "          pos: {}", self.pos)?;
        Ok(())
    }
}

impl std::error::Error for SemanticError {}

/* --- Global --------------------------------------------- */

pub fn check_def_global(def: &mut Definition, table: &SymbolTable) -> Result<(), SemanticError> {
    match def {
        Definition::TypeDefinition(_) => Ok(()),
        Definition::ProcedureDefinition(p) => check_def_proc(p, table),
    }
}

fn check_def_proc(
    proc: &mut ProcedureDefinition,
    table: &SymbolTable,
) -> Result<(), SemanticError> {
    let local_table: &SymbolTable = &table
        .lookup(&proc.name, None)
        .and_then(|e| {
            if let Entry::ProcedureEntry(e) = e {
                Some(e)
            } else {
                None
            }
        })
        .expect("check_def_proc: given procedure not in symbol table")
        .local_table;

    proc.body
        .iter_mut()
        .try_for_each(|s| check_statement(s, local_table, table))?;

    Ok(())
}

/* --- Statements ----------------------------------------- */

fn check_statement(
    statement: &mut Statement,
    table: &SymbolTable,
    global_table: &SymbolTable,
) -> Result<(), SemanticError> {
    match statement {
        Statement::IfStatement(s) => {
            let cond_expr_type = check_expression(&mut s.condition, table, global_table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    _msg: format!("IfConditionMustBeBoolean: {s:?}"),
                    //pos: s.condition.pos,
                });
            }

            check_statement(&mut s.then_branch, table, global_table)?;

            if let Some(ref mut s) = s.else_branch {
                check_statement(s, table, global_table)?;
            }

            Ok(())
        }

        Statement::EmptyStatement => Ok(()),

        Statement::CallStatement(s) => {
            let proc = global_table.lookup(&s.name, None).ok_or(SemanticError {
                _msg: format!("UndefinedIdentifier: {s:?}"),
                //pos: s.pos,
            })?;
            let Entry::ProcedureEntry(proc) = proc else {
                return Err(SemanticError {
                    _msg: format!("CallOfNonProcedure: {proc:?} {s:?}"),
                    //pos: s.pos,
                });
            };

            if s.arguments.len() != proc.parameter_types.len() {
                return Err(SemanticError {
                    _msg: format!(
                        "ArgumentCountMismatch: {:?} {:?}",
                        s.arguments, proc.parameter_types
                    ),
                    //pos: s.pos,
                });
            }

            for (i, (arg, param)) in s
                .arguments
                .iter_mut()
                .zip(proc.parameter_types.iter())
                .enumerate()
            {
                let arg_type = check_expression(arg, table, global_table)?;
                if arg_type != &param.typ {
                    return Err(SemanticError {
                        _msg: format!(
                            "ArgumentTypeMismatch: {}(): arg {i}: [{param:?}] {arg:?}",
                            s.name
                        ),
                        //pos: arg.pos,
                    });
                }
                if param.is_reference && !arg.is_variable() {
                    return Err(SemanticError {
                        _msg: format!(
                            "ArgumentMustBeAVariable: {}(): arg {i}: [{param:?}] {arg:?}",
                            s.name
                        ),
                        //pos: arg.pos,
                    });
                }
            }

            Ok(())
        }

        Statement::WhileStatement(s) => {
            let cond_expr_type = check_expression(&mut s.condition, table, global_table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    _msg: format!("WhileConditionMustBeBoolean: {s:?}"),
                    //pos: s.condition.pos,
                });
            }

            check_statement(&mut s.body, table, global_table)?;

            Ok(())
        }

        Statement::AssignStatement(s) => {
            let target_type = check_variable(&mut s.target, table, global_table)?;
            let value_type = check_expression(&mut s.value, table, global_table)?;

            if target_type.is_array() || target_type != value_type {
                return Err(SemanticError {
                    _msg: format!("IllegalAssignment: {s:?}"),
                    //pos: s.pos,
                });
            }

            Ok(())
        }

        Statement::CompoundStatement(s) => s
            .iter_mut()
            .try_for_each(|s| check_statement(s, table, global_table)),
    }
}

/* --- Expressions ---------------------------------------- */

fn check_expression<'a>(
    expr: &'a mut Expression,
    table: &'a SymbolTable,
    global_table: &'a SymbolTable,
) -> Result<&'a Type, SemanticError> {
    match expr {
        Expression::BinaryExpression(expr) => {
            let left_type = check_expression(&mut expr.left, table, global_table)?;
            let right_type = check_expression(&mut expr.right, table, global_table)?;

            let Some(result_type) = expr.operator.result_type(left_type, right_type) else {
                return Err(SemanticError {
                    _msg: format!("OperandTypeMismatch: {expr:?}"),
                    //pos: expr.pos,
                });
            };

            Ok(result_type)
        }
        Expression::UnaryExpression(expr) => {
            let right_type = check_expression(&mut expr.operand, table, global_table)?;

            let Some(result_type) = expr.operator.result_type(right_type) else {
                return Err(SemanticError {
                    _msg: format!("OperandTypeMismatch: {expr:?}"),
                    //pos: expr.pos,
                });
            };

            Ok(result_type)
        }
        Expression::IntLiteral(_) => Ok(&Type::PrimitiveType(PrimitiveType::Int)),
        Expression::VariableExpression(var) => check_variable(var, table, global_table),
    }
}

fn check_variable<'a>(
    var: &mut Variable,
    table: &'a SymbolTable,
    global_table: &'a SymbolTable,
) -> Result<&'a Type, SemanticError> {
    match var {
        Variable::NamedVariable(var_name) => {
            let entry = table
                .lookup(var_name, Some(global_table))
                .ok_or(SemanticError {
                    _msg: format!("UndefinedIdentifier: {var:?}"),
                    //pos: var.pos,
                })?;
            let Entry::VariableEntry(entry) = entry else {
                return Err(SemanticError {
                    _msg: format!("NotAVariable: {entry:?} {var:?}"),
                    //pos: var.pos,
                });
            };

            Ok(&entry.typ)
        }
        Variable::ArrayAccess(arr_acc) => {
            let array_type = check_variable(&mut arr_acc.array, table, global_table)?;
            let Type::ArrayType(array_type) = array_type else {
                return Err(SemanticError {
                    _msg: format!("IndexingNonArray: {var:?}"),
                    //pos: arr_acc.pos,
                });
            };

            arr_acc.typ = Some(array_type.clone());

            let index_type = check_expression(&mut arr_acc.index, table, global_table)?;
            if !index_type.is_int() {
                return Err(SemanticError {
                    _msg: format!("IndexTypeMismatch: {var:?}"),
                    //pos: arr_acc.pos,
                });
            }

            Ok(&*array_type.base_type)
        }
    }
}
