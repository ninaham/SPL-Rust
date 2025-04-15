pub mod build_symbol_table;
pub mod table_initializer;
mod utils;

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
    _msg: &'static str,
    //pos: i64,
}

/* --- Global --------------------------------------------- */

pub fn check_def_global(def: &Definition, table: &SymbolTable) -> Result<(), SemanticError> {
    match def {
        Definition::TypeDefinition(_) => Ok(()),
        Definition::ProcedureDefinition(p) => check_def_proc(p, table),
    }
}

fn check_def_proc(proc: &ProcedureDefinition, table: &SymbolTable) -> Result<(), SemanticError> {
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
        .iter()
        .try_for_each(|s| check_statement(s, local_table, table))?;

    Ok(())
}

/* --- Statements ----------------------------------------- */

fn check_statement(
    statement: &Statement,
    table: &SymbolTable,
    global_table: &SymbolTable,
) -> Result<(), SemanticError> {
    match statement {
        Statement::IfStatement(s) => {
            let cond_expr_type = check_expression(&s.condition, table, global_table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    _msg: "IfConditionMustBeBoolean",
                    //pos: s.condition.pos,
                });
            }

            check_statement(&s.then_branch, table, global_table)?;

            if let Some(ref s) = s.else_branch {
                check_statement(s, table, global_table)?;
            }

            Ok(())
        }

        Statement::EmptyStatement => Ok(()),

        Statement::CallStatement(s) => {
            let proc = global_table.lookup(&s.name, None).ok_or(SemanticError {
                _msg: "UndefinedIdentifier",
                //pos: s.pos,
            })?;
            let Entry::ProcedureEntry(proc) = proc else {
                return Err(SemanticError {
                    _msg: "CallOfNonProcedure",
                    //pos: s.pos,
                });
            };

            if s.arguments.len() != proc.parameter_types.len() {
                return Err(SemanticError {
                    _msg: "ArgumentCountMismatch",
                    //pos: s.pos,
                });
            }

            for (arg, param) in s.arguments.iter().zip(proc.parameter_types.iter()) {
                let arg_type = check_expression(arg, table, global_table)?;
                if arg_type != &param.typ {
                    return Err(SemanticError {
                        _msg: "ArgumentTypeMismatch",
                        //pos: arg.pos,
                    });
                }
                if param.is_reference && arg.is_variable() {
                    return Err(SemanticError {
                        _msg: "ArgumentMustBeAVariable",
                        //pos: arg.pos,
                    });
                }
            }

            Ok(())
        }

        Statement::WhileStatement(s) => {
            let cond_expr_type = check_expression(&s.condition, table, global_table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    _msg: "WhileConditionMustBeBoolean",
                    //pos: s.condition.pos,
                });
            }

            check_statement(&s.body, table, global_table)?;

            Ok(())
        }

        Statement::AssignStatement(s) => {
            let target_type = check_variable(&s.target, table, global_table)?;
            let value_type = check_expression(&s.value, table, global_table)?;

            if target_type.is_array() || target_type != value_type {
                return Err(SemanticError {
                    _msg: "IllegalAssignment",
                    //pos: s.pos,
                });
            }

            Ok(())
        }

        Statement::CompoundStatement(s) => s
            .iter()
            .try_for_each(|s| check_statement(s, table, global_table)),
    }
}

/* --- Expressions ---------------------------------------- */

fn check_expression<'a>(
    expr: &Expression,
    table: &'a SymbolTable,
    global_table: &'a SymbolTable,
) -> Result<&'a Type, SemanticError> {
    match expr {
        Expression::BinaryExpression(expr) => {
            let left_type = check_expression(&expr.left, table, global_table)?;
            let right_type = check_expression(&expr.right, table, global_table)?;

            if left_type != right_type {
                return Err(SemanticError {
                    _msg: "OperandTypeMismatch",
                    //pos: expr.pos,
                });
            }
            // TODO: also check if operands' types match operator

            Ok(left_type)
        }
        Expression::UnaryExpression(expr) => {
            let operand_type = check_expression(&expr.operand, table, global_table)?;
            // TODO: check if operand's type matches operator
            Ok(operand_type)
        }
        Expression::IntLiteral(_) => Ok(&Type::PrimitiveType(PrimitiveType::Int)),
        Expression::VariableExpression(var) => check_variable(var, table, global_table),
    }
}

fn check_variable<'a>(
    var: &Variable,
    table: &'a SymbolTable,
    global_table: &'a SymbolTable,
) -> Result<&'a Type, SemanticError> {
    match var {
        Variable::NamedVariable(var_name) => {
            let entry = table
                .lookup(var_name, Some(global_table))
                .ok_or(SemanticError {
                    _msg: "UndefinedIdentifier",
                    //pos: var.pos,
                })?;
            let Entry::VariableEntry(entry) = entry else {
                return Err(SemanticError {
                    _msg: "NotAVariable",
                    //pos: var.pos,
                });
            };

            Ok(&entry.typ)
        }
        Variable::ArrayAccess(arr_acc) => {
            let array_type = check_variable(&arr_acc.array, table, global_table)?;
            let Type::ArrayType(array_type) = array_type else {
                return Err(SemanticError {
                    _msg: "IndexingNonArray",
                    //pos: arr_acc.pos,
                });
            };

            let index_type = check_expression(&arr_acc.index, table, global_table)?;
            if !index_type.is_int() {
                return Err(SemanticError {
                    _msg: "IndexTypeMismatch",
                    //pos: arr_acc.pos,
                });
            }

            Ok(&*array_type.base_type)
        }
    }
}
