//use parser::parser::alphanumeric;

use std::fs;

//use cli::CLI_INPUT;
use parser::parse_everything_else::parse;
use semant::{SemanticError, build_symbol_table::build_symbol_table, check_def_global};

pub mod absyn;
pub mod cli;
pub mod parser;
pub mod semant;
pub mod table;

fn main() -> Result<(), SemanticError> {
    for entry in fs::read_dir("./spl-testfiles/runtime_tests/").unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        println!("parsing {}", file_name.to_str().unwrap());
        let test = fs::read_to_string(format!(
            "./spl-testfiles/runtime_tests/{}",
            file_name.to_str().unwrap()
        ))
        .unwrap();
        let n = parse(test.as_str());
        let table = build_symbol_table(&n)?;
        if let Err(err) = n
            .definitions
            .iter()
            .try_for_each(|def| check_def_global(def, &table))
        {
            println!("{err:?}\n");
        }
        //println!("{:#?}", n);
        //println!("{:#?}", *CLI_INPUT);
    }

    Ok(())
}
