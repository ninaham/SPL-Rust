use std::{collections::HashMap, rc::Rc, sync::Mutex};

use crate::table::{
    entry::{Entry, ParameterType, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::{PrimitiveType, Type},
};

pub fn init_symbol_table(s_t: Rc<Mutex<SymbolTable>>) {
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

    let mut symbol_table = s_t.lock().unwrap();

    for t in types {
        symbol_table
            .enter(
                t.to_string(),
                Entry::TypeEntry(TypeEntry {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                }),
            )
            .unwrap()
    }

    for (name, types) in procedures {
        symbol_table
            .enter(
                name.to_string(),
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),
                        upper_level: Some(Rc::downgrade(&s_t)),
                    },
                    parameter_types: types,
                }),
            )
            .unwrap()
    }
}
