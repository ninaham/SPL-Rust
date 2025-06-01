use std::{collections::HashMap, rc::Rc, sync::Mutex};

use crate::table::{
    entry::{Entry, Parameter, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::{PrimitiveType, Type},
};

const TYPES: [&str; 1] = ["int"];

const PROCEDURES: [(&str, &[Parameter]); 10] = [
    (
        "printi",
        &[Parameter {
            typ: Type::PrimitiveType(PrimitiveType::Int),
            is_reference: false,
        }],
    ),
    (
        "printc",
        &[Parameter {
            typ: Type::PrimitiveType(PrimitiveType::Int),
            is_reference: false,
        }],
    ),
    (
        "readi",
        &[Parameter {
            typ: Type::PrimitiveType(PrimitiveType::Int),
            is_reference: true,
        }],
    ),
    (
        "readc",
        &[Parameter {
            typ: Type::PrimitiveType(PrimitiveType::Int),
            is_reference: true,
        }],
    ),
    ("exit", &[]),
    (
        "time",
        &[Parameter {
            typ: Type::PrimitiveType(PrimitiveType::Int),
            is_reference: true,
        }],
    ),
    (
        "clearAll",
        &[Parameter {
            typ: Type::PrimitiveType(PrimitiveType::Int),
            is_reference: false,
        }],
    ),
    (
        "setPixel",
        &[
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
        &[
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
        &[
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

pub fn init_symbol_table(s_t: &Rc<Mutex<SymbolTable>>) {
    let mut symbol_table = s_t.lock().unwrap();

    for t in TYPES {
        symbol_table
            .enter(
                t.to_string(),
                Entry::TypeEntry(TypeEntry {
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                }),
            )
            .unwrap();
    }

    for (name, params) in PROCEDURES {
        symbol_table
            .enter(
                name.to_string(),
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),
                        upper_level: Some(Rc::downgrade(s_t)),
                    },
                    parameters: params.into(),
                }),
            )
            .unwrap();
    }
}
