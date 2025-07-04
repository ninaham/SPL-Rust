pub mod build_symbol_table;
pub mod table_initializer;
mod utils;

use std::{cell::RefCell, fmt, rc::Rc};

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
    pub msg: String,
    //pos: i64,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SemanticError: \"{}\"", self.msg)?;
        //write!(f, "          pos: {}", self.pos)?;
        Ok(())
    }
}

impl std::error::Error for SemanticError {}

/* --- Global --------------------------------------------- */

pub fn check_def_global(
    def: &mut Definition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(), SemanticError> {
    match def {
        Definition::TypeDefinition(_) => Ok(()),
        Definition::ProcedureDefinition(p) => check_def_proc(p, table),
    }
}

fn check_def_proc(
    proc: &mut ProcedureDefinition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(), SemanticError> {
    let table = table.borrow();
    let local_table: &SymbolTable = &table
        .lookup(&proc.name)
        .and_then(|e| {
            if let Entry::ProcedureEntry(e) = e {
                Some(e)
            } else {
                None
            }
        })
        .expect("check_def_proc: given procedure not in symbol table")
        .local_table;
    drop(table);
    proc.body
        .iter_mut()
        .try_for_each(|s| check_statement(s, local_table))?;

    Ok(())
}

/* --- Statements ----------------------------------------- */

fn check_statement(statement: &mut Statement, table: &SymbolTable) -> Result<(), SemanticError> {
    match statement {
        Statement::IfStatement(s) => {
            let cond_expr_type = check_expression(&mut s.condition, table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    msg: format!("IfConditionMustBeBoolean: {s:?}"),
                    //pos: s.condition.pos,
                });
            }

            check_statement(&mut s.then_branch, table)?;

            if let Some(ref mut s) = s.else_branch {
                check_statement(s, table)?;
            }

            Ok(())
        }

        Statement::EmptyStatement => Ok(()),

        Statement::CallStatement(s) => {
            let proc = table.lookup(&s.name).ok_or_else(|| SemanticError {
                msg: format!("UndefinedIdentifier: {s:?}"),
                //pos: s.pos,
            })?;
            let Entry::ProcedureEntry(proc) = proc else {
                return Err(SemanticError {
                    msg: format!("CallOfNonProcedure: {proc:?} {s:?}"),
                    //pos: s.pos,
                });
            };

            if s.arguments.len() != proc.parameters.len() {
                return Err(SemanticError {
                    msg: format!(
                        "ArgumentCountMismatch: {:?} {:?}",
                        s.arguments, proc.parameters
                    ),
                    //pos: s.pos,
                });
            }

            for (i, (arg, param)) in s
                .arguments
                .iter_mut()
                .zip(proc.parameters.iter())
                .enumerate()
            {
                let arg_type = check_expression(arg, table)?;
                if arg_type != param.typ {
                    return Err(SemanticError {
                        msg: format!(
                            "ArgumentTypeMismatch: {}(): arg {i}: [{param:?}] {arg:?}",
                            s.name
                        ),
                        //pos: arg.pos,
                    });
                }
                if param.is_reference && !arg.is_variable() {
                    return Err(SemanticError {
                        msg: format!(
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
            let cond_expr_type = check_expression(&mut s.condition, table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    msg: format!("WhileConditionMustBeBoolean: {s:?}"),
                    //pos: s.condition.pos,
                });
            }

            check_statement(&mut s.body, table)?;

            Ok(())
        }

        Statement::AssignStatement(s) => {
            let target_type = check_variable(&mut s.target, table)?;
            let value_type = check_expression(&mut s.value, table)?;

            if target_type.is_array() || target_type != value_type {
                return Err(SemanticError {
                    msg: format!("IllegalAssignment: {s:?}"),
                    //pos: s.pos,
                });
            }

            Ok(())
        }

        Statement::CompoundStatement(s) => s.iter_mut().try_for_each(|s| check_statement(s, table)),
    }
}

/* --- Expressions ---------------------------------------- */

fn check_expression<'a>(
    expr: &'a mut Expression,
    table: &'a SymbolTable,
) -> Result<Type, SemanticError> {
    match expr {
        Expression::BinaryExpression(expr) => {
            let left_type = check_expression(&mut expr.left, table)?;
            let right_type = check_expression(&mut expr.right, table)?;

            let Some(result_type) = expr.operator.result_type(&left_type, &right_type) else {
                return Err(SemanticError {
                    msg: format!("OperandTypeMismatch: {expr:?}"),
                    //pos: expr.pos,
                });
            };

            Ok(result_type)
        }
        Expression::UnaryExpression(expr) => {
            let right_type = check_expression(&mut expr.operand, table)?;

            let Some(result_type) = expr.operator.result_type(&right_type) else {
                return Err(SemanticError {
                    msg: format!("OperandTypeMismatch: {expr:?}"),
                    //pos: expr.pos,
                });
            };

            Ok(result_type)
        }
        Expression::IntLiteral(_) => Ok(Type::PrimitiveType(PrimitiveType::Int)),
        Expression::VariableExpression(var) => check_variable(var, table),
    }
}

fn check_variable(var: &mut Variable, table: &SymbolTable) -> Result<Type, SemanticError> {
    match var {
        Variable::NamedVariable(var_name) => {
            let entry = table.lookup(var_name).ok_or_else(|| SemanticError {
                msg: format!("UndefinedIdentifier: {var:?}"),
                //pos: var.pos,
            })?;
            let Entry::VariableEntry(entry) = entry else {
                return Err(SemanticError {
                    msg: format!("NotAVariable: {entry:?} {var:?}"),
                    //pos: var.pos,
                });
            };

            Ok(entry.typ)
        }
        Variable::ArrayAccess(arr_acc) => {
            let array_type = check_variable(&mut arr_acc.array, table)?;
            let Type::ArrayType(array_type) = array_type else {
                return Err(SemanticError {
                    msg: format!("IndexingNonArray: {var:?}"),
                    //pos: arr_acc.pos,
                });
            };

            arr_acc.typ = Some(array_type.clone());

            let index_type = check_expression(&mut arr_acc.index, table)?;
            if !index_type.is_int() {
                return Err(SemanticError {
                    msg: format!("IndexTypeMismatch: {var:?}"),
                    //pos: arr_acc.pos,
                });
            }

            Ok(*array_type.base_type)
        }
    }
}
