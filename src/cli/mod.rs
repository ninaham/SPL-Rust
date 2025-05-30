use std::io::{IsTerminal, Write};
use std::path::Path;
use std::process::Stdio;
use std::{fs::File, process};

use anyhow::{anyhow, bail, Ok};
use clap::{arg, ArgGroup, Command, Id};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::{
    base_blocks::BlockGraph,
    code_gen::Tac,
    optimizations::dead_code_elimination,
    optimizations::live_variables::LiveVariables,
    optimizations::reaching_expressions::ReachingDefinitions,
    optimizations::worklist::{Definition, Worklist},
    parser::parse_everything_else::parse,
    semant::{build_symbol_table::build_symbol_table, check_def_global},
    table::entry::Entry,
};

#[expect(clippy::cognitive_complexity)]
pub fn load_program_data() -> Command {
    Command::new("SPl Rust Compiler")
        .version("0.1.0")
        .args([
            arg!(file: <file> "Path to SPL code"),
            arg!(parse: -p --parse "Parse input file, returns the abstract syntax tree"),
            arg!(tables: -t --tables "Fills symbol tables and prints them"),
            arg!(semant: -s --semant "Semantic analysis"),
            arg!(tac: -'3' --tac "Generates three address code"),
            arg!(proc: -P --proc <name> "Name of the procedure to be examined"),
            arg!(dot: -d --dot ["output"] "Generates block graph").require_equals(true),
            arg!(rch: -r --rch "Reaching Definitions"),
            arg!(lv: -l --lv "Live Variables"),
            arg!(dead: -e --dce ["output"] "Dead Code Eliminiation").require_equals(true),
        ])
        .group(
            ArgGroup::new("phase")
                .required(false)
                .multiple(false)
                .args([
                    "parse", "tables", "semant", "tac", "dot", "rch", "lv", "dead",
                ]),
        )
}

#[expect(clippy::too_many_lines)]
pub fn process_matches(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let file = matches.get_one::<String>("file").unwrap();
    let input = std::fs::read_to_string(file)?.leak();

    let Some(phase) = matches.get_one::<Id>("phase") else {
        bail!("Gode Generation for ECO32 not yet implemented")
    };

    let mut absyn = parse(input)?;

    if phase == "parse" {
        println!("{absyn:#?}");
        return Ok(());
    }

    let table = build_symbol_table(&absyn)?;

    if phase == "tables" {
        println!("{table:#?}");
        return Ok(());
    }

    absyn
        .definitions
        .iter_mut()
        .try_for_each(|def| check_def_global(def, &table))?;

    if phase == "semant" {
        return Ok(());
    }

    let mut address_code = Tac::new(table.clone());
    address_code.code_generation(&absyn);

    if phase == "tac" {
        println!("{address_code}");
        return Ok(());
    }

    let graphs = address_code.proc_table.keys().collect::<Vec<_>>();
    let theme = ColorfulTheme::default();

    let sel_proc = matches
        .get_one::<String>("proc")
        .and_then(|phase_arg| graphs.iter().position(|p| p == &phase_arg));

    let sel_proc = if let Some(sel_proc) = sel_proc {
        sel_proc
    } else {
        Select::with_theme(&theme)
            .with_prompt("Which procedure?")
            .items(&graphs)
            .default(0)
            .interact()?
    };
    let mut graph = BlockGraph::from_tac(address_code.proc_table.get(graphs[sel_proc]).unwrap());

    graph.common_subexpression_elimination(&table.lock().unwrap());

    if phase == "dot" {
        let mut filename = format!("{}.dot", graphs[sel_proc]);
        let outputname = format!("as file: {filename}");
        let outputs = vec!["print", &outputname, "dot Tx11", "xdot"];

        let output = matches.get_one::<String>("dot").and_then(|arg| {
            if Path::new(arg)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dot"))
            {
                filename = arg.to_string();
                Some(1)
            } else {
                outputs.iter().position(|o| o == arg)
            }
        });

        let output = if let Some(output) = output {
            output
        } else if std::io::stdout().is_terminal() {
            Select::with_theme(&theme)
                .with_prompt("Which output mode?")
                .items(&outputs)
                .default(0)
                .interact()?
        } else {
            0 // always write dot code to stdout if not terminal
        };

        match output {
            0 => {
                println!("{graph}");
            }
            1 => {
                let mut file = File::create(filename)?;
                writeln!(file, "{graph}")?;
            }
            2 => {
                process::Command::new("dot")
                    .arg("--version")
                    .output()
                    .map(|output| output.status.success())
                    .map_err(|_| anyhow!("dot is not installed!".red()))?;

                let mut dot = process::Command::new("dot")
                    .arg("-Tx11")
                    .stdin(Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = dot.stdin.as_mut() {
                    write!(stdin, "{graph}")?;
                }
                dot.wait()?;
            }
            3 => {
                process::Command::new("xdot")
                    .arg("-h")
                    .output()
                    .map(|output| output.status.success())
                    .map_err(|_| anyhow!("xdot is not installed".red()))?;

                let mut xdot = process::Command::new("xdot")
                    .arg("-")
                    .stdin(Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = xdot.stdin.as_mut() {
                    write!(stdin, "{graph}")?;
                }
                xdot.wait()?;
            }
            _ => {
                println!("No valid input");
            }
        }

        return Ok(());
    }

    if phase == "lv" {
        let proc_name = graphs[sel_proc];
        let proc_def = table.lock().unwrap().lookup(proc_name);
        let Some(Entry::ProcedureEntry(proc_def)) = proc_def else {
            unreachable!()
        };
        let live_variables = LiveVariables::run(&mut graph, &proc_def.local_table);

        println!("Variables:");
        for (i, v) in live_variables.defs.iter().enumerate() {
            println!("{i:>5}: {v}");
        }
        println!();

        let col_width = live_variables.defs.len();
        println!(
            "{:>5} {:<col_width$} {:<col_width$} {:<col_width$} {:<col_width$}",
            "Block", "DEF", "USE", "LIVin", "LIVout",
        );
        for (n, (((g, p), i), o)) in live_variables
            .def
            .iter()
            .zip(live_variables.use_bits)
            .zip(live_variables.livin)
            .zip(live_variables.livout)
            .enumerate()
        {
            println!(
                "{n:>5} {} {} {} {}",
                g.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
                p.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
                i.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
                o.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
            );
        }

        return Ok(());
    }

    if phase == "dead" {
        let mut filename = format!("{}.dot", graphs[sel_proc]);
        let outputname = format!("as file: {filename}");
        let outputs = vec!["print", &outputname, "dot Tx11", "xdot"];

        let proc_name = graphs[sel_proc];
        let proc_def = table.lock().unwrap().lookup(proc_name);
        let Some(Entry::ProcedureEntry(proc_def)) = proc_def else {
            unreachable!()
        };
        let live_variables = LiveVariables::run(&mut graph, &proc_def.local_table);

        graph = dead_code_elimination::dead_code_elimination(&graph, &live_variables);

        let output = matches.get_one::<String>("dot").and_then(|arg| {
            if Path::new(arg)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("dot"))
            {
                filename = arg.to_string();
                Some(1)
            } else {
                outputs.iter().position(|o| o == arg)
            }
        });

        let output = if let Some(output) = output {
            output
        } else if std::io::stdout().is_terminal() {
            Select::with_theme(&theme)
                .with_prompt("Which output mode?")
                .items(&outputs)
                .default(0)
                .interact()?
        } else {
            0 // always write dot code to stdout if not terminal
        };

        match output {
            0 => {
                println!("{graph}");
            }
            1 => {
                let mut file = File::create(filename)?;
                writeln!(file, "{graph}")?;
            }
            2 => {
                process::Command::new("dot")
                    .arg("--version")
                    .output()
                    .map(|output| output.status.success())
                    .map_err(|_| anyhow!("dot is not installed!".red()))?;

                let mut dot = process::Command::new("dot")
                    .arg("-Tx11")
                    .stdin(Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = dot.stdin.as_mut() {
                    write!(stdin, "{graph}")?;
                }
                dot.wait()?;
            }
            3 => {
                process::Command::new("xdot")
                    .arg("-h")
                    .output()
                    .map(|output| output.status.success())
                    .map_err(|_| anyhow!("xdot is not installed".red()))?;

                let mut xdot = process::Command::new("xdot")
                    .arg("-")
                    .stdin(Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = xdot.stdin.as_mut() {
                    write!(stdin, "{graph}")?;
                }
                xdot.wait()?;
            }
            _ => {
                println!("No valid input");
            }
        }

        return Ok(());
    }

    if phase == "rch" {
        let proc_name = graphs[sel_proc];
        let proc_def = table.lock().unwrap().lookup(proc_name);
        let Some(Entry::ProcedureEntry(proc_def)) = proc_def else {
            unreachable!()
        };
        let reaching_definitions = ReachingDefinitions::run(&mut graph, &proc_def.local_table);

        println!("Definitions:");
        println!(
            "{}",
            Definition::fmt_table(reaching_definitions.defs.iter())
        );
        println!();

        let col_width = reaching_definitions.defs.len();
        println!(
            "{:>5} {:<col_width$} {:<col_width$} {:<col_width$} {:<col_width$}",
            "Block", "GEN", "PRSV", "RCHin", "RCHout",
        );
        for (n, (((g, p), i), o)) in reaching_definitions
            .gen_bits
            .iter()
            .zip(reaching_definitions.prsv)
            .zip(reaching_definitions.rchin)
            .zip(reaching_definitions.rchout)
            .enumerate()
        {
            println!(
                "{n:>5} {} {} {} {}",
                g.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
                p.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
                i.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
                o.iter()
                    .map(|b| b.then_some('1').unwrap_or('0'))
                    .collect::<String>(),
            );
        }

        return Ok(());
    }

    unreachable!()
}
