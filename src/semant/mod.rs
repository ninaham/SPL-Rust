// Declare submodules used for building and initializing the symbol table
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

// Represents a semantic error with a message (and optional position info)
#[derive(Debug)]
pub struct SemanticError {
    pub msg: String,
    // Position could be added for better error tracking
    // pos: i64,
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SemanticError: \"{}\"", self.msg)?;
        // write!(f, "          pos: {}", self.pos)?;
        Ok(())
    }
}

impl std::error::Error for SemanticError {}

/* --- Global Declarations and Checks --------------------- */

// Check a global definition (either type or procedure)
pub fn check_def_global(
    def: &mut Definition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(), SemanticError> {
    match def {
        Definition::TypeDefinition(_) => Ok(()), // Type definitions need no further checking here
        Definition::ProcedureDefinition(p) => check_def_proc(p, table), // Check procedure body
    }
}

// Check a full procedure definition and validate its body
fn check_def_proc(
    proc: &mut ProcedureDefinition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(), SemanticError> {
    // Get the corresponding local symbol table from the procedure entry
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

    // Check all statements in the procedure body
    proc.body
        .iter_mut()
        .try_for_each(|s| check_statement(s, local_table))?;

    Ok(())
}

/* --- Statement Checks ----------------------------------- */

// Check a single statement (e.g., if, while, call, assign, compound, etc.)
fn check_statement(statement: &mut Statement, table: &SymbolTable) -> Result<(), SemanticError> {
    match statement {
        Statement::IfStatement(s) => {
            // Ensure the condition is of type boolean
            let cond_expr_type = check_expression(&mut s.condition, table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    msg: format!("IfConditionMustBeBoolean: {s:?}"),
                });
            }

            // Check both branches of the if-statement
            check_statement(&mut s.then_branch, table)?;
            if let Some(ref mut s) = s.else_branch {
                check_statement(s, table)?;
            }

            Ok(())
        }

        Statement::EmptyStatement => Ok(()), // No check needed for empty statements

        Statement::CallStatement(s) => {
            // Check if the procedure exists in the symbol table
            let proc = table.lookup(&s.name).ok_or_else(|| SemanticError {
                msg: format!("UndefinedIdentifier: {s:?}"),
            })?;

            // Ensure it's actually a procedure entry
            let Entry::ProcedureEntry(proc) = proc else {
                return Err(SemanticError {
                    msg: format!("CallOfNonProcedure: {proc:?} {s:?}"),
                });
            };

            // Check the argument count
            if s.arguments.len() != proc.parameters.len() {
                return Err(SemanticError {
                    msg: format!(
                        "ArgumentCountMismatch: {:?} {:?}",
                        s.arguments, proc.parameters
                    ),
                });
            }

            // Check types of each argument and ensure reference arguments are variables
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
                    });
                }
                if param.is_reference && !arg.is_variable() {
                    return Err(SemanticError {
                        msg: format!(
                            "ArgumentMustBeAVariable: {}(): arg {i}: [{param:?}] {arg:?}",
                            s.name
                        ),
                    });
                }
            }

            Ok(())
        }

        Statement::WhileStatement(s) => {
            // Check that while condition is boolean
            let cond_expr_type = check_expression(&mut s.condition, table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    msg: format!("WhileConditionMustBeBoolean: {s:?}"),
                });
            }

            // Check the loop body
            check_statement(&mut s.body, table)?;

            Ok(())
        }

        Statement::AssignStatement(s) => {
            // Get types of both sides of the assignment
            let target_type = check_variable(&mut s.target, table)?;
            let value_type = check_expression(&mut s.value, table)?;

            // Ensure they are the same and not assigning to an array
            if target_type.is_array() || target_type != value_type {
                return Err(SemanticError {
                    msg: format!("IllegalAssignment: {s:?}"),
                });
            }

            Ok(())
        }

        Statement::CompoundStatement(s) =>
        // Check all statements inside compound (block) statement
        {
            s.iter_mut().try_for_each(|s| check_statement(s, table))
        }
    }
}

/* --- Expression Checks ---------------------------------- */

// Check an expression and return its resulting type
fn check_expression<'a>(
    expr: &'a mut Expression,
    table: &'a SymbolTable,
) -> Result<Type, SemanticError> {
    match expr {
        Expression::BinaryExpression(expr) => {
            // Recursively check both operands
            let left_type = check_expression(&mut expr.left, table)?;
            let right_type = check_expression(&mut expr.right, table)?;

            // Validate and get resulting type from the operator
            let Some(result_type) = expr.operator.result_type(&left_type, &right_type) else {
                return Err(SemanticError {
                    msg: format!("OperandTypeMismatch: {expr:?}"),
                });
            };

            Ok(result_type)
        }
        Expression::UnaryExpression(expr) => {
            let right_type = check_expression(&mut expr.operand, table)?;

            let Some(result_type) = expr.operator.result_type(&right_type) else {
                return Err(SemanticError {
                    msg: format!("OperandTypeMismatch: {expr:?}"),
                });
            };

            Ok(result_type)
        }
        Expression::IntLiteral(_) => Ok(Type::PrimitiveType(PrimitiveType::Int)), // Constant int
        Expression::VariableExpression(var) => check_variable(var, table), // Delegate to variable check
    }
}

/* --- Variable Checks ------------------------------------ */

// Check that a variable is defined and return its type
fn check_variable(var: &mut Variable, table: &SymbolTable) -> Result<Type, SemanticError> {
    match var {
        Variable::NamedVariable(var_name) => {
            // Lookup variable in symbol table
            let entry = table.lookup(var_name).ok_or_else(|| SemanticError {
                msg: format!("UndefinedIdentifier: {var:?}"),
            })?;

            // Ensure it's a variable
            let Entry::VariableEntry(entry) = entry else {
                return Err(SemanticError {
                    msg: format!("NotAVariable: {entry:?} {var:?}"),
                });
            };

            Ok(entry.typ)
        }

        Variable::ArrayAccess(arr_acc) => {
            // Ensure the array is an actual array type
            let array_type = check_variable(&mut arr_acc.array, table)?;
            let Type::ArrayType(array_type) = array_type else {
                return Err(SemanticError {
                    msg: format!("IndexingNonArray: {var:?}"),
                });
            };

            // Store the type in the AST node for future use
            arr_acc.typ = Some(array_type.clone());

            // Check that the index is an integer
            let index_type = check_expression(&mut arr_acc.index, table)?;
            if !index_type.is_int() {
                return Err(SemanticError {
                    msg: format!("IndexTypeMismatch: {var:?}"),
                });
            }

            Ok(*array_type.base_type)
        }
    }
}
