use anyhow::{Ok, bail};
use clap::{ArgGroup, Command, Id, arg};

use crate::{
    code_gen::Tac,
    parser::parse_everything_else::parse,
    semant::{build_symbol_table::build_symbol_table, check_def_global},
};

pub fn load_program_data() -> Command {
    Command::new("SPl Rust Compiler")
        .version("0.1.0")
        .args([
            arg!(file: <file> "Path to input graph"),
            arg!(parse: -p --parse "Parse input file, returns the abstract syntax tree"),
            arg!(tables: -t --tables "Fills symbol tables and prints them"),
            arg!(semant: -s --semant "Semantic analysis"),
            arg!(tac: -'3' --tac "Generates three address code"),
        ])
        .group(
            ArgGroup::new("phase")
                .required(false)
                .multiple(false)
                .args(["parse", "tables", "semant", "tac"]),
        )
}

pub fn process_matches(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let file = matches.get_one::<String>("file").unwrap();
    let input = std::fs::read_to_string(file)?.leak();

    let Some(phase) = matches.get_one::<Id>("phase") else {
        bail!("Gode Generation for ECO32 not yet implemented")
    };

    let mut address_code;

    let mut absyn = parse(input)?;
    if phase == "parse" {
        println!("{:#?}", absyn);
        return Ok(());
    }
    let table = build_symbol_table(&absyn)?;
    if phase == "tables" {
        println!("{:?}", table);
        return Ok(());
    }

    absyn
        .definitions
        .iter_mut()
        .try_for_each(|def| check_def_global(def, &table.clone()))?;

    if phase == "semant" {
        return Ok(());
    }
    address_code = Tac::new(&table);
    address_code.code_generation(&absyn);
    if phase == "tac" {
        println!("{}", address_code);
        return Ok(());
    }

    unreachable!()
}
