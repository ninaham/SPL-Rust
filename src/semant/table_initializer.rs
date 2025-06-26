use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::table::{
    entry::{Entry, Parameter, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::{PrimitiveType, Type},
};

const TYPES: [&str; 1] = ["int"];

fn preocedures<'a>() -> Vec<(&'a str, Vec<Parameter>)> {
    vec![
        (
            "printi",
            vec![Parameter {
                name: "".to_string(),
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: false,
            }],
        ),
        (
            "printc",
            vec![Parameter {
                name: "".to_string(),
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: false,
            }],
        ),
        (
            "readi",
            vec![Parameter {
                name: "".to_string(),
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: true,
            }],
        ),
        (
            "readc",
            vec![Parameter {
                name: "".to_string(),
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: true,
            }],
        ),
        ("exit", vec![]),
        (
            "time",
            vec![Parameter {
                name: "".to_string(),
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: true,
            }],
        ),
        (
            "clearAll",
            vec![Parameter {
                name: "".to_string(),
                typ: Type::PrimitiveType(PrimitiveType::Int),
                is_reference: false,
            }],
        ),
        (
            "setPixel",
            vec![
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
            ],
        ),
        (
            "drawLine",
            vec![
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
            ],
        ),
        (
            "drawCircle",
            vec![
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
                Parameter {
                    name: "".to_string(),
                    typ: Type::PrimitiveType(PrimitiveType::Int),
                    is_reference: false,
                },
            ],
        ),
    ]
}

pub fn init_symbol_table(s_t: &Rc<RefCell<SymbolTable>>) {
    let mut symbol_table = s_t.borrow_mut();

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

    for (name, params) in preocedures() {
        symbol_table
            .enter(
                name.to_string(),
                Entry::ProcedureEntry(ProcedureEntry {
                    local_table: SymbolTable {
                        entries: HashMap::new(),
                        upper_level: Some(Rc::downgrade(s_t)),
                    },
                    parameters: params,
                }),
            )
            .unwrap();
    }
}
