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
    pub parameter_types: Vec<ParameterType>,
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
pub struct ParameterType {
    pub typ: Type,
    pub is_reference: bool,
}
