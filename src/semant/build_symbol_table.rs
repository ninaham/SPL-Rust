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

/// Builds the global symbol table from the given program.
///
/// Initializes a new global symbol table, inserts built-in types and procedures,
/// then iterates over all top-level definitions in the program and adds them
/// to the symbol table.
/// Returns a reference-counted pointer to the global symbol table or a SemanticError.
pub fn build_symbol_table(program: &Program) -> Result<Rc<RefCell<SymbolTable>>, SemanticError> {
    // Create a new empty global symbol table wrapped in Rc<RefCell> for shared mutability
    let global_table = Rc::new(RefCell::new(SymbolTable::new()));

    // Initialize the symbol table with built-in types and procedures
    table_initializer::init_symbol_table(&global_table);

    // Iterate over all definitions in the program and enter them into the global symbol table
    program
        .definitions
        .iter()
        .try_for_each(|def| enter_global_def(def, &global_table))?;

    // Return the initialized global symbol table
    Ok(global_table)
}

/// Enters a global definition (procedure or type) into the symbol table.
///
/// Matches on the definition type and delegates to the appropriate handler.
/// Updates the symbol table accordingly or returns a SemanticError.
pub fn enter_global_def(
    def: &Definition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(), SemanticError> {
    match def {
        // If it's a procedure definition, process and insert it
        Definition::ProcedureDefinition(procedure_definition) => {
            let (name, entry) = enter_procedure_def(procedure_definition, table)?;
            let mut t = table.borrow_mut();
            t.enter(name, entry)?;
        }
        // If it's a type definition, process and insert it
        Definition::TypeDefinition(type_definition) => {
            let mut t = table.borrow_mut();
            let (name, entry) = enter_type_def(type_definition, &t)?;
            t.enter(name, entry)?;
        }
    }

    Ok(())
}

/// Processes a type definition and returns the corresponding symbol table entry.
///
/// Converts the type expression to a concrete Type and wraps it in a TypeEntry.
/// Returns the name and entry for insertion into the symbol table.
pub fn enter_type_def(
    def: &TypeDefinition,
    table: &SymbolTable,
) -> Result<(String, Entry), SemanticError> {
    // Convert the abstract syntax type expression into an internal Type representation
    let entry = TypeEntry {
        typ: type_expression_to_type(&def.type_expression, table)?,
    };

    Ok((def.name.clone(), Entry::TypeEntry(entry)))
}

/// Processes a procedure definition and returns the corresponding symbol table entry.
///
/// Creates a new local symbol table for the procedure, enters parameters and variables,
/// and collects parameter metadata. Returns the name and ProcedureEntry.
pub fn enter_procedure_def(
    def: &ProcedureDefinition,
    table: &Rc<RefCell<SymbolTable>>,
) -> Result<(String, Entry), SemanticError> {
    // Create a new local symbol table for the procedure with a reference to the global one
    let mut local_table = SymbolTable {
        entries: HashMap::new(),
        upper_level: Some(Rc::downgrade(table)),
    };

    // Insert all parameters into the local symbol table
    def.parameters
        .iter()
        .try_for_each(|def| enter_param_def(def, &mut local_table))?;

    // Collect parameter metadata (name, type, reference status)
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

    // Insert all local variables into the procedure's local symbol table
    def.variables
        .iter()
        .try_for_each(|def| enter_var_def(def, &mut local_table))?;

    // Construct the procedure entry with its local symbol table and parameters
    let entry = ProcedureEntry {
        local_table,
        parameters,
    };

    Ok((def.name.clone(), Entry::ProcedureEntry(entry)))
}

/// Converts a type expression (abstract syntax) to a concrete Type representation.
///
/// Handles named types and array types by recursively resolving base types.
/// Returns an error if the type is unknown or invalid.
pub fn type_expression_to_type(
    type_ex: &TypeExpression,
    table: &SymbolTable,
) -> Result<Type, SemanticError> {
    match type_ex {
        // For array types, recursively resolve the base type and wrap it with size info
        TypeExpression::ArrayTypeExpression(array_type_expression) => {
            Ok(Type::ArrayType(ArrayType {
                base_type: Box::new(type_expression_to_type(
                    &array_type_expression.base_type,
                    table,
                )?),
                size: array_type_expression.array_size,
            }))
        }
        // For named types, look them up in the symbol table
        TypeExpression::NamedTypeExpression(nte) => {
            let Some(entry) = table.lookup(nte) else {
                return Err(SemanticError {
                    msg: format!("Type {nte} not found"),
                });
            };
            // Validate that the entry is actually a type
            match entry {
                Entry::TypeEntry(type_entry) => Ok(type_entry.typ),
                _ => Err(SemanticError {
                    msg: format!("{nte} is not a type"),
                }),
            }
        }
    }
}

/// Enters a variable definition into the symbol table.
///
/// Converts the variable's type expression to a Type and inserts a VariableEntry.
/// Variables are not references by default.
pub fn enter_var_def(
    def: &VariableDefinition,
    table: &mut SymbolTable,
) -> Result<(), SemanticError> {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, table)?,
        is_reference: false,
    };

    table.enter(def.name.clone(), Entry::VariableEntry(entry))?;

    Ok(())
}

/// Enters a parameter definition into the symbol table.
///
/// Converts the parameter's type expression to a Type and inserts a VariableEntry.
/// The parameter's reference flag is preserved.
pub fn enter_param_def(
    def: &ParameterDefinition,
    table: &mut SymbolTable,
) -> Result<(), SemanticError> {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, table)?,
        is_reference: def.is_reference,
    };

    table.enter(def.name.clone(), Entry::VariableEntry(entry))?;

    Ok(())
}
