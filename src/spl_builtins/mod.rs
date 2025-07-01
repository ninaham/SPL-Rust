use std::{cell::Cell, cell::RefMut, time::Instant};

use crate::{
    interpreter::value::{Value, ValueRef},
    table::{entry::Parameter, types::Type},
};

pub const NAMED_TYPES: [&str; 1] = ["int"];

thread_local! {
    static START_TIME: Cell<Instant> = unreachable!("START_TIME not initialized!");
}
pub fn init_start_time() {
    START_TIME.set(Instant::now());
}

builtin_procedures! {
    proc printi(i: int) {
        print!("{i}");
    }
    proc printc(c: int) {
        let c = u8::try_from(c).unwrap_or_else(|_| panic!("Argument to printc() should be a valid ASCII value: {c}")) as char;
        print!("{c}");
    }
    proc readi(ref i: int)
    proc readc(ref c: int)
    proc exit()
    proc time(ref t: int) {
        *t = START_TIME.get().elapsed().as_secs().try_into().unwrap();
    }
    proc clearAll(c: int)
    proc setPixel(a: int, b: int, c: int)
    proc drawLine(a: int, b: int, c: int, d: int, e: int)
    proc drawCircle(a: int, b: int, c: int, d: int)
}

macro_rules! builtin_procedures {
    {
        $(
            proc $proc_name:ident($($param_ref:ident $($param_name:ident)?: $param_type:ident),*)
            $( $proc_impl:block )?
        )*
    } => {
        pub const PROCEDURES: [(&str, &[Parameter], Option<&(dyn Fn(&[ValueRef<'_>]) + 'static)>); count!($($proc_name)*)] = [
            $(
                {
                    static PARAMS: [Parameter; count!($($param_type)*)] = [ $( builtin_procedures!(@param $param_ref $($param_name)?: $param_type) ),* ];
                    (
                        stringify!($proc_name),
                        PARAMS.as_slice(),
                        builtin_procedures!(@impl | $($param_ref $($param_name)?: $param_type),* | $($proc_impl)?),
                    )
                },
            )*
        ];
    };

    (@param     $name:ident: $type:ident) => { Parameter::new(String::new(), builtin_procedures!(@type $type), false) };
    (@param ref $name:ident: $type:ident) => { Parameter::new(String::new(), builtin_procedures!(@type $type), true ) };

    (@type int) => { Type::INT };

    (@impl |$($ref:ident $($name:ident)?: $type:ident),*|             ) => { None };
    (@impl |                                            | $body:block ) => { Some(&|_| $body) };
    (@impl |$($ref:ident $($name:ident)?: $type:ident),+| $body:block ) => {
        Some(&|args| {
            let mut args = args.iter();
            $(
                builtin_procedures!(@arg args $type($ref $($name)?));
            )+
            $body
        })
    };

    (@arg $args:ident $type:ident(    $name:ident)) => {
        let builtin_procedures!(@arg_type $type($name)) = *$args.next().unwrap().borrow() else { unreachable!() };
    };
    (@arg $args:ident $type:ident(ref $name:ident)) => {
        let mut $name = RefMut::map($args.next().unwrap().borrow_mut(), |v| {
            let builtin_procedures!(@arg_type $type(v)) = v else { unreachable!() };
            v
        });
    };

    (@arg_type int ($name:ident)) => { Value::Int ($name) };
    (@arg_type bool($name:ident)) => { Value::Bool($name) };
}
use builtin_procedures;

macro_rules! count {
    () => { 0 };
    ($x:tt $($xs:tt)*) => { 1 + count!($($xs)*) };
}
use count;
