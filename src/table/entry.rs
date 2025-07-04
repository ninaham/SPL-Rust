#![expect(clippy::enum_variant_names)]

use super::{symbol_table::SymbolTable, types::Type};

#[derive(Debug, Clone)]
pub enum Entry {
    ProcedureEntry(ProcedureEntry),
    VariableEntry(VariableEntry),
    TypeEntry(TypeEntry),
}

#[derive(Debug, Clone)]
pub struct ProcedureEntry {
    pub local_table: SymbolTable,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct VariableEntry {
    pub typ: Type,
    pub is_reference: bool,
}

#[derive(Debug, Clone)]
pub struct TypeEntry {
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub typ: Type,
    pub is_reference: bool,
}

impl Parameter {
    pub const fn new(name: String, typ: Type, is_reference: bool) -> Self {
        Self {
            name,
            typ,
            is_reference,
        }
    }
}
