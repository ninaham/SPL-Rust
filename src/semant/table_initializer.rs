use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::table::{
    entry::{Entry, ProcedureEntry, TypeEntry},
    symbol_table::SymbolTable,
    types::Type,
};

const TYPES: [&str; 1] = ["int"];

builtin_procedures! {
    proc printi(i: int);
    proc printc(c: int);
    proc readi(ref i: int);
    proc readc(ref c: int);
    proc exit();
    proc time(ref t: int);
    proc clearAll(c: int);
    proc setPixel(a: int, b: int, c: int);
    proc drawLine(a: int, b: int, c: int, d: int, e: int);
    proc drawCircle(a: int, b: int, c: int, d: int);
}

pub fn init_symbol_table(s_t: &Rc<RefCell<SymbolTable>>) {
    let mut symbol_table = s_t.borrow_mut();

    for t in TYPES {
        symbol_table
            .enter(
                t.to_string(),
                Entry::TypeEntry(TypeEntry { typ: Type::INT }),
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
                    parameters: params.to_vec(),
                }),
            )
            .unwrap();
    }
}

macro_rules! builtin_procedures {
    {
        $(
            proc $proc_name:ident($($arg_ref:ident $($arg_name:ident)?: $arg_type:ident),*);
        )*
    } => {
        const PROCEDURES: [(&str, &[$crate::table::entry::Parameter]); count!($($proc_name)*)] = [
            $(
                {
                    static PARAMS: [$crate::table::entry::Parameter; count!($($arg_type)*)] = [ $( builtin_procedures!(@arg $arg_ref $($arg_name)?: $arg_type) ),* ];
                    (stringify!($proc_name), PARAMS.as_slice())
                },
            )*
        ];
    };

    (@arg     $name:ident: $type:ident) => { $crate::table::entry::Parameter::new(std::string::String::new(), builtin_procedures!(@type $type), false) };
    (@arg ref $name:ident: $type:ident) => { $crate::table::entry::Parameter::new(std::string::String::new(), builtin_procedures!(@type $type), true ) };

    (@type int) => { Type::INT };
}
use builtin_procedures;

macro_rules! count {
    () => { 0 };
    ($x:tt $($xs:tt)*) => { 1 + count!($($xs)*) };
}
use count;
