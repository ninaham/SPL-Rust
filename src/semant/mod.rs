mod utils;

use crate::absyn::{
    absyn::{Definition, Expression, Statement, TypeExpression, Variable},
    procedure_definition::ProcedureDefinition,
};

pub struct SemanticError {
    msg: &'static str,
    pos: i64,
}

/* --- Global --------------------------------------------- */

fn check_def_global(def: &Definition, table: &SymbolTable) -> Result<(), SemanticError> {
    match def {
        Definition::TypeDefinition(_) => Ok(()),
        Definition::ProcedureDefinition(p) => check_def_proc(&p, table),
    }
}

fn check_def_proc(proc: &ProcedureDefinition, table: &SymbolTable) -> Result<(), SemanticError> {
    let local_table: &SymbolTable = table.get(proc.name)?.local_table?;

    proc.body.iter().for_each(|s| {
        check_statement(s, local_table);
    });

    Ok(())
}

/* --- Statements ----------------------------------------- */

fn check_statement(statement: &Statement, table: &SymbolTable) -> Result<(), SemanticError> {
    match statement {
        Statement::IfStatement(s) => {
            let cond_expr_type = check_expression(&s.condition, table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    msg: "IfConditionMustBeBoolean",
                    pos: s.condition.pos,
                });
            }

            check_statement(&s.then_branch, table)?;

            if let Some(ref s) = s.else_branch {
                check_statement(s, table)?;
            }

            Ok(())
        }

        Statement::EmptyStatement => Ok(()),

        Statement::CallStatement(s) => {
            let proc = table.lookup(&s.name, s.pos)?;
            let SymbolEntry::Procedure(proc) = proc else {
                return Err(SemanticError {
                    msg: "CallOfNonProcedure",
                    pos: s.pos,
                });
            };

            if s.arguments.len() != proc.parameters.len() {
                return Err(SemanticError {
                    msg: "ArgumentCountMismatch",
                    pos: s.pos,
                });
            }

            for (arg, param) in s.arguments.iter().zip(proc.parameters.iter()) {
                let arg_type = check_expression(&arg, table)?;
                if arg_type != param.r#type {
                    return Err(SemanticError {
                        msg: "ArgumentTypeMismatch",
                        pos: arg.pos,
                    });
                }
                if param.is_ref && arg.is_variable() {
                    return Err(SemanticError {
                        msg: "ArgumentMustBeAVariable",
                        pos: arg.pos,
                    });
                }
            }

            Ok(())
        }

        Statement::WhileStatement(s) => {
            let cond_expr_type = check_expression(&s.condition, table)?;
            if !cond_expr_type.is_bool() {
                return Err(SemanticError {
                    msg: "WhileConditionMustBeBoolean",
                    pos: s.condition.pos,
                });
            }

            check_statement(&s.body, table)?;

            Ok(())
        }

        Statement::AssignStatement(s) => {
            let target_type = check_variable(&s.target, table)?;
            let value_type = check_expression(&s.value, table)?;

            if target_type.is_array() || target_type != value_type {
                return Err(SemanticError {
                    msg: "IllegalAssignment",
                    pos: s.pos,
                });
            }

            Ok(())
        }

        Statement::CompoundStatement(s) => s.iter().try_for_each(|s| check_statement(s, table)),
    }
}

/* --- Expressions ---------------------------------------- */

fn check_expression(
    expr: &Expression,
    table: &SymbolTable,
) -> Result<TypeExpression, SemanticError> {
    match expr {
        Expression::BinaryExpression(expr) => {
            let left_type = check_expression(&expr.left, table)?;
            let right_type = check_expression(&expr.right, table)?;

            if left_type != right_type {
                return Err(SemanticError {
                    msg: "OperandTypeMismatch",
                    pos: expr.pos,
                });
            }
            // TODO: also check if operands' types match operator

            Ok(left_type)
        }
        Expression::UnaryExpression(expr) => {
            let operand_type = check_expression(&expr.operand, table)?;
            // TODO: check if operand's type matches operator
            Ok(operand_type)
        }
        Expression::IntLiteral(_) => Ok(TypeExpression::NamedTypeExpression("int".to_string())),
        Expression::VariableExpression(var) => check_variable(var, table),
    }
}

fn check_variable(var: &Variable, table: &SymbolTable) -> Result<TypeExpression, SemanticError> {
    match var {
        Variable::NamedVariable(var_name) => {
            let entry = table.lookup(var_name, var.pos)?;
            let SymbolEntry::Variable(entry) = entry else {
                return Err(SemanticError {
                    msg: "NotAVariable",
                    pos: var.pos,
                });
            };

            Ok(entry.r#type)
        }
        Variable::ArrayAccess(arr_acc) => {
            let array_type = check_variable(&arr_acc.array, table)?;
            let TypeExpression::ArrayTypeExpression(ref array_type) = array_type else {
                return Err(SemanticError {
                    msg: "IndexingNonArray",
                    pos: arr_acc.pos,
                });
            };

            let index_type = check_expression(&arr_acc.index, table)?;
            if !index_type.is_int() {
                return Err(SemanticError {
                    msg: "IndexTypeMismatch",
                    pos: arr_acc.pos,
                });
            }

            Ok(array_type.base_type)
        }
    }
}
