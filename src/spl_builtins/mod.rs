use crate::{
    interpreter::value::{Value, ValueRef},
    table::{entry::Parameter, types::Type},
};

pub const NAMED_TYPES: [&str; 1] = ["int"];

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
    proc time(ref t: int)
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
                let builtin_procedures!(@arg_type $type($ref $($name)?)) = *args.next().unwrap().borrow() else { unreachable!() };
            )+
            $body
        })
    };

    (@arg_type int ($arg:tt)) => { Value::Int (builtin_procedures!(@param_name $arg)) };
    (@arg_type bool($arg:tt)) => { Value::Bool(builtin_procedures!(@param_name $arg)) };

    (@param_name ref $name:ident) => { $name };
    (@param_name     $name:ident) => { $name };
}
use builtin_procedures;

macro_rules! count {
    () => { 0 };
    ($x:tt $($xs:tt)*) => { 1 + count!($($xs)*) };
}
use count;
