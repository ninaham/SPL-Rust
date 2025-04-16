use std::collections::HashMap;

use crate::table::{
    entry::{Entry, ParameterType, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::{PrimitiveType, Type},
};

impl SymbolTable {
    pub fn init(&mut self) {
        let types = ["int"];
        let procedures = [
            (
                "printi",
                vec![ParameterType {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                }],
            ),
            (
                "printc",
                vec![ParameterType {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                }],
            ),
            (
                "readi",
                vec![ParameterType {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: true,
                }],
            ),
            (
                "readc",
                vec![ParameterType {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: true,
                }],
            ),
            ("exit", vec![]),
            (
                "time",
                vec![ParameterType {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: true,
                }],
            ),
            (
                "clearAll",
                vec![ParameterType {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                }],
            ),
            (
                "setPixel",
                vec![
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                ],
            ),
            (
                "drawLine",
                vec![
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                ],
            ),
            (
                "drawCircle",
                vec![
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                    ParameterType {
                        typ: Type::PrimitiveType(PrimitiveType::Int),
                        is_reference: false,
                    },
                ],
            ),
        ];

        for t in types {
            self.enter(
                t.to_string(),
                Entry::TypeEntry(TypeEntry {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                }),
            )
            .unwrap()
        }

        for (name, types) in procedures {
            self.enter(
                name.to_string(),
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),
                    },
                    parameter_types: types,
                }),
            )
            .unwrap()
        }
    }
}
