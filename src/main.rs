//use parser::parser::alphanumeric;

use std::fs;

use parser::parse_everything_else::parse;

pub mod absyn;
pub mod parser;
pub mod tokens;

fn main() {
    let test = "proc fak(n: int, ref res: int) {
    if (n = 1) res := 1;
    else {
        fak(n-1, res);
        res:=res*n;
    }
}

proc main() {
    var i: int;
    var j: int;
    i := 5;
    fak(i, j);
    printi(j);
}";
    let n = parse(test);
    println!("{:?}", n);
}
