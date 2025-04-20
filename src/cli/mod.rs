use std::io::Write;
use std::process::Stdio;
use std::{fs::File, process};

use anyhow::{Ok, bail};
use clap::{ArgGroup, Command, Id, arg};
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    base_blocks::BlockGraph,
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
            arg!(dot: -d --dot "Generates block graph"),
        ])
        .group(
            ArgGroup::new("phase")
                .required(false)
                .multiple(false)
                .args(["parse", "tables", "semant", "tac", "dot"]),
        )
}

pub fn process_matches(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let file = matches.get_one::<String>("file").unwrap();
    let input = std::fs::read_to_string(file)?.leak();

    let Some(phase) = matches.get_one::<Id>("phase") else {
        bail!("Gode Generation for ECO32 not yet implemented")
    };

    let mut absyn = parse(input)?;

    if phase == "parse" {
        println!("{:#?}", absyn);
        return Ok(());
    }

    let table = build_symbol_table(&absyn)?;

    if phase == "tables" {
        println!("{:#?}", table);
        return Ok(());
    }

    absyn
        .definitions
        .iter_mut()
        .try_for_each(|def| check_def_global(def, &table.clone()))?;

    if phase == "semant" {
        return Ok(());
    }

    let mut address_code = Tac::new(&table);
    address_code.code_generation(&absyn);

    if phase == "tac" {
        println!("{}", address_code);
        return Ok(());
    }

    if phase == "dot" {
        let graphs: Vec<&String> = address_code.proc_table.keys().clone().collect();
        let theme = ColorfulTheme::default();
        let sel_graph = Select::with_theme(&theme)
            .with_prompt("Which procedure?")
            .items(&graphs)
            .default(0)
            .interact()?;
        let filename = format!("{}.dot", graphs[sel_graph]);
        let outputname = format!("as file: {}", filename);
        let outputs = vec!["print", &outputname, "xdot"];
        let output = Select::with_theme(&theme)
            .with_prompt("Which output mode?")
            .items(&outputs)
            .default(0)
            .interact()?;
        let graph = BlockGraph::from_tac(address_code.proc_table.get(graphs[sel_graph]).unwrap());

        match output {
            0 => {
                println!("{}", graph);
            }
            1 => {
                let mut file = File::create(filename)?;
                writeln!(file, "{}", graph)?;
            }
            2 => {
                let mut xdot = process::Command::new("xdot")
                    .arg("-")
                    .stdin(Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = xdot.stdin.as_mut() {
                    stdin.write_all(format!("{}", graph).as_bytes())?;
                }
                xdot.wait()?;
            }
            _ => {
                println!("No valid input");
            }
        }

        return Ok(());
    }

    unreachable!()
}
