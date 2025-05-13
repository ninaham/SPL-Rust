use std::{collections::HashMap, rc::Rc, sync::Mutex};

use crate::table::{
    entry::{Entry, Parameter, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::{PrimitiveType, Type},
};

pub fn init_symbol_table(s_t: Rc<Mutex<SymbolTable>>) {
    let types = ["int"];
    let procedures = [
        (
            "printi",
            vec![Parameter {
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: false,
            }],
        ),
        (
            "printc",
            vec![Parameter {
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: false,
            }],
        ),
        (
            "readi",
            vec![Parameter {
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: true,
            }],
        ),
        (
            "readc",
            vec![Parameter {
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: true,
            }],
        ),
        ("exit", vec![]),
        (
            "time",
            vec![Parameter {
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: true,
            }],
        ),
        (
            "clearAll",
            vec![Parameter {
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: false,
            }],
        ),
        (
            "setPixel",
            vec![
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
            ],
        ),
        (
            "drawLine",
            vec![
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
            ],
        ),
        (
            "drawCircle",
            vec![
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
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

    for (name, params) in procedures {
        symbol_table
            .enter(
                name.to_string(),
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),
                        upper_level: Some(Rc::downgrade(&s_t)),
                    },
                    parameters: params,
                }),
            )
            .unwrap()
    }
}
