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

pub fn build_symbol_table(program: &Program) -> SymbolTable {
    let mut global_table = SymbolTable {
        entries: HashMap::new(),
    };

    global_table.init();

    program
        .definitions
        .iter()
        .for_each(|def| enter_global_def(def, &mut global_table));

    global_table
}

pub fn enter_global_def(def: &Definition, table: &mut SymbolTable) {
    match def {
        Definition::ProcedureDefinition(procedure_definition) => {
            let (name, entry) = enter_procedure_def(procedure_definition, table);
            table.enter(name, entry).unwrap();
        }
        Definition::TypeDefinition(type_definition) => {
            let (name, entry) = enter_type_def(type_definition, table);
            table.enter(name, entry).unwrap();
        }
    }
}

pub fn enter_type_def(def: &TypeDefinition, table: &SymbolTable) -> (String, Entry) {
    let entry = TypeEntry {
        typ: type_expression_to_type(&def.type_expression, table),
    };

    (def.name.clone(), Entry::TypeEntry(entry))
}

pub fn enter_procedure_def(def: &ProcedureDefinition, table: &SymbolTable) -> (String, Entry) {
    let mut local_table = SymbolTable {
        entries: HashMap::new(),
    };

    def.parameters
        .iter()
        .for_each(|def| enter_param_def(def, &mut local_table, table));

    let parameter_types = def
        .parameters
        .iter()
        .map(|param| ParameterType {
            typ: type_expression_to_type(&param.type_expression, table),
            is_reference: param.is_reference,
        })
        .collect::<Vec<ParameterType>>();

    def.variales
        .iter()
        .for_each(|def| enter_var_def(def, &mut local_table, table));

    let entry = ProcedureEntry {
        local_table,
        parameter_types,
    };

    (def.name.clone(), Entry::ProcedureEntry(entry))
}

pub fn type_expression_to_type(type_ex: &TypeExpression, table: &SymbolTable) -> Type {
    match type_ex {
        TypeExpression::ArrayTypeExpression(array_type_expression) => Type::ArrayType(ArrayType {
            base_type: Box::new(type_expression_to_type(
                &array_type_expression.base_type,
                table,
            )),
            size: array_type_expression.array_size,
        }),
        TypeExpression::NamedTypeExpression(nte) => {
            let entry = table.lookup(nte, None).unwrap().clone();
            match entry {
                Entry::TypeEntry(type_entry) => type_entry.typ,
                _ => todo!(),
            }
        }
    }
}

pub fn enter_var_def(
    def: &VariableDefinition,
    table: &mut SymbolTable,
    global_table: &SymbolTable,
) {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, global_table),
        is_reference: false,
    };

    table
        .enter(def.name.clone(), Entry::VariableEntry(entry))
        .unwrap();
}

pub fn enter_param_def(
    def: &ParameterDefinition,
    table: &mut SymbolTable,
    global_table: &SymbolTable,
) {
    let entry = VariableEntry {
        typ: type_expression_to_type(&def.type_expression, global_table),
        is_reference: def.is_reference,
    };

    table
        .enter(def.name.clone(), Entry::VariableEntry(entry))
        .unwrap();
}
