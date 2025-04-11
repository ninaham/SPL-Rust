//use parser::parser::alphanumeric;

use std::fs;

use cli::CLI_INPUT;
use parser::parse_everything_else::parse;

pub mod absyn;
pub mod cli;
pub mod parser;

fn main() {
    for entry in fs::read_dir("./spl-testfiles/runtime_tests/").unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        println!("parsing {}", file_name.to_str().unwrap());
        let test = fs::read_to_string(format!(
            "./spl-testfiles/runtime_tests/{}",
            file_name.to_str().unwrap()
        ))
        .unwrap();
        let _n = parse(test.as_str());
        //println!("{:#?}", n);
        println!("{:#?}", *CLI_INPUT);
    }
}
