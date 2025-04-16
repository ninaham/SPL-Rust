use std::collections::HashMap;

use crate::{
    absyn::{
        absyn::{Definition, Program, TypeExpression},
        parameter_definition::ParameterDefinition,
        procedure_definition::ProcedureDefinition,
        type_definition::TypeDefinition,
        variable_definition::VariableDefinition,
    },
    table::{
        entry::{Entry, ParameterType, ProcedureEntry, TypeEntry, VariableEntry},
        symbol_table::SymbolTable,
        types::{ArrayType, Type},
    },
};

use super::SemanticError;

pub fn build_symbol_table(program: &Program) -> Result<SymbolTable, SemanticError> {
    let mut global_table = SymbolTable {
        entries: HashMap::new(),
    };

    global_table.init();

    program
        .definitions
        .iter()
        .try_for_each(|def| enter_global_def(def, &mut global_table))?;

    Ok(global_table)
}

pub fn enter_global_def(def: &Definition, table: &mut SymbolTable) -> Result<(), SemanticError> {
    match def {
        Definition::ProcedureDefinition(procedure_definition) => {
            let (name, entry) = enter_procedure_def(procedure_definition, table)?;
            table.enter(name, entry)?;
        }
        Definition::TypeDefinition(type_definition) => {
            let (name, entry) = enter_type_def(type_definition, table)?;
            table.enter(name, entry)?;
        }
    }

    Ok(())
}

pub fn enter_type_def(
    def: &TypeDefinition,
    table: &SymbolTable,
) -> Result<(String, Entry), SemanticError> {
    let entry = TypeEntry {
        typ: type_expression_to_type(&def.type_expression, table)?,
    };

    Ok((def.name.clone(), Entry::TypeEntry(entry)))
}

pub fn enter_procedure_def(
    def: &ProcedureDefinition,
    table: &SymbolTable,
) -> Result<(String, Entry), SemanticError> {
    let mut local_table = SymbolTable {
        entries: HashMap::new(),
    };

    def.parameters
        .iter()
        .try_for_each(|def| enter_param_def(def, &mut local_table, table))?;

    let parameter_types = def
        .parameters
        .iter()
        .map(|param| {
            let param_type = match type_expression_to_type(&param.type_expression, table) {
                Ok(typ) => typ,
                Err(err) => return Err(err),
            };
            Ok(ParameterType {
                typ: param_type,
                is_reference: param.is_reference,
            })
        })
        .collect::<Result<Vec<ParameterType>, SemanticError>>()?;

    def.variales
        .iter()
        .try_for_each(|def| enter_var_def(def, &mut local_table, table))?;

    let entry = ProcedureEntry {
        local_table,
        parameter_types,
    };

    Ok((def.name.clone(), Entry::ProcedureEntry(entry)))
}

pub fn type_expression_to_type(
    type_ex: &TypeExpression,
    table: &SymbolTable,
) -> Result<Type, SemanticError> {
    match type_ex {
        TypeExpression::ArrayTypeExpression(array_type_expression) => {
            Ok(Type::ArrayType(ArrayType {
                base_type: Box::new(type_expression_to_type(
                    &array_type_expression.base_type,
                    table,
                )?),
                size: array_type_expression.array_size,
            }))
        }
        TypeExpression::NamedTypeExpression(nte) => {
            let entry = match table.lookup(nte, None) {
                Some(entry) => entry.clone(),
                None => {
                    return Err(SemanticError {
                        _msg: format!("Type {} not found", nte),
                    });
                }
            };
            match entry {
                Entry::TypeEntry(type_entry) => Ok(type_entry.typ),
                _ => Err(SemanticError {
                    _msg: format!("{} is not a type", nte),
                }),
            }
        }
    }
}

pub fn enter_var_def(
    def: &VariableDefinition,
    table: &mut SymbolTable,
    global_table: &SymbolTable,
) -> Result<(), SemanticError> {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, global_table)?,
        is_reference: false,
    };

    table.enter(def.name.clone(), Entry::VariableEntry(entry))?;

    Ok(())
}

pub fn enter_param_def(
    def: &ParameterDefinition,
    table: &mut SymbolTable,
    global_table: &SymbolTable,
) -> Result<(), SemanticError> {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, global_table)?,
        is_reference: def.is_reference,
    };

    table.enter(def.name.clone(), Entry::VariableEntry(entry))?;

    Ok(())
}
