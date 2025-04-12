use super::{symbol_table::SymbolTable, types::Type};

#[derive(Debug)]
pub enum Entry {
    ProcedureEntry(ProcedureEntry),
    VariableEntry(VariableEntry),
    TypeEntry(TypeEntry),
}

#[derive(Debug)]
pub struct ProcedureEntry {
    pub local_table: SymbolTable,
    pub parameter_types: Vec<ParameterType>,
}

#[derive(Debug)]
pub struct VariableEntry {
    pub typ: Type,
    pub is_reference: bool,
}

#[derive(Debug)]
pub struct TypeEntry {
    pub typ: Type,
}

#[derive(Debug)]
pub struct ParameterType {
    pub typ: Type,
    pub is_reference: bool,
}
