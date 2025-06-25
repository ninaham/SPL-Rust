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
}

#[derive(Debug, Clone)]
pub struct TypeEntry {
    pub typ: Type,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub typ: Type,
    pub is_reference: bool,
}
