use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    absyn::{
        absyn::{Definition, Program, TypeExpression},
        parameter_definition::ParameterDefinition,
        procedure_definition::ProcedureDefinition,
        type_definition::TypeDefinition,
        variable_definition::VariableDefinition,
    },
    table::{
        entry::{Entry, Parameter, ProcedureEntry, TypeEntry, VariableEntry},
        symbol_table::SymbolTable,
        types::{ArrayType, Type},
    },
};

use super::{SemanticError, table_initializer};

pub fn build_symbol_table(program: &Program) -> Result<Rc<RefCell<SymbolTable>>, SemanticError> {
    let global_table = SymbolTable {
        entries: HashMap::new(),
        upper_level: None,
    };

    let global_table = Rc::new(RefCell::new(global_table));

    table_initializer::init_symbol_table(&global_table);

    program
        .definitions
        .iter()
        .try_for_each(|def| enter_global_def(def, &global_table))?;

    Ok(global_table)
}

pub fn enter_global_def(
    def: &Definition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(), SemanticError> {
    match def {
        Definition::ProcedureDefinition(procedure_definition) => {
            let (name, entry) = enter_procedure_def(procedure_definition, table)?;
            let mut t = table.borrow_mut();
            t.enter(name, entry)?;
        }
        Definition::TypeDefinition(type_definition) => {
            let mut t = table.borrow_mut();
            let (name, entry) = enter_type_def(type_definition, &t)?;
            t.enter(name, entry)?;
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
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(String, Entry), SemanticError> {
    let mut local_table = SymbolTable {
        entries: HashMap::new(),
        upper_level: Some(Rc::downgrade(table)),
    };

    def.parameters
        .iter()
        .try_for_each(|def| enter_param_def(def, &mut local_table))?;

    let parameters = def
        .parameters
        .iter()
        .map(|param| {
            let param_type = match type_expression_to_type(&param.type_expression, &local_table) {
                Ok(typ) => typ,
                Err(err) => return Err(err),
            };
            Ok(Parameter {
                name: param.name.clone(),
                typ: param_type,
                is_reference: param.is_reference,
            })
        })
        .collect::<Result<Vec<Parameter>, SemanticError>>()?;

    def.variables
        .iter()
        .try_for_each(|def| enter_var_def(def, &mut local_table))?;

    let entry = ProcedureEntry {
        local_table,
        parameters,
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
            let Some(entry) = table.lookup(nte) else {
                return Err(SemanticError {
                    msg: format!("Type {nte} not found"),
                });
            };
            match entry {
                Entry::TypeEntry(type_entry) => Ok(type_entry.typ),
                _ => Err(SemanticError {
                    msg: format!("{nte} is not a type"),
                }),
            }
        }
    }
}

pub fn enter_var_def(
    def: &VariableDefinition,
    table: &mut SymbolTable,
) -> Result<(), SemanticError> {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, table)?,
    };

    table.enter(def.name.clone(), Entry::VariableEntry(entry))?;

    Ok(())
}

pub fn enter_param_def(
    def: &ParameterDefinition,
    table: &mut SymbolTable,
) -> Result<(), SemanticError> {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, table)?,
    };

    table.enter(def.name.clone(), Entry::VariableEntry(entry))?;

    Ok(())
}
