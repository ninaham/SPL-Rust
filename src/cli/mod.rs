use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{self, Display, Write as _};
use std::io::{IsTerminal, Write as _};
use std::path::Path;
use std::process::Stdio;
use std::rc::Rc;
use std::{fs::File, process};

use anyhow::{anyhow, bail};
use bitvec::vec::BitVec;
use clap::{ArgGroup, Command, Id, arg};
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::interpreter::definition_evaluator::start_main;
use crate::interpreter::tac_interpreter::eval_tac;
use crate::{
    base_blocks::BlockGraph,
    code_gen::Tac,
    optimizations::constant_propagation::ConstantPropagation,
    optimizations::live_variables::LiveVariables,
    optimizations::reaching_expressions::ReachingDefinitions,
    optimizations::worklist::{Lattice, Worklist},
    parser::parse_everything_else::parse,
    semant::{build_symbol_table::build_symbol_table, check_def_global},
    table::entry::Entry,
    table::symbol_table::SymbolTable,
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
            arg!(interpret: -i --interpret "SPL Interpreter"),
            arg!(interprettac: -I --interprettac "TAC Interpreter"),
            arg!(tac: -'3' --tac "Generates three address code"),
            arg!(proc: -P --proc <name> "Name of the procedure to be examined"),
            arg!(optis: -O --optis <optis> "Optimizations to apply: [cse, rch, lv, dead, gcp, scc, licm]")
                .num_args(1..)
                .value_delimiter(','),
            arg!(dot: -d --dot ["output"] "Generates block graph").require_equals(true),
            arg!(optimization: -o --optimize "All optimizations"),
        ])
        .group(
            ArgGroup::new("phase")
                .required(false)
                .multiple(false)
                .args(["parse", "tables", "semant", "interpret", "interprettac", "tac", "dot"]),
        )
}

pub fn process_matches(matches: &clap::ArgMatches) -> anyhow::Result<()> {
    let file = matches.get_one::<String>("file").unwrap();
    let input = std::fs::read_to_string(file)?.leak();

    let mut absyn = parse(input)?;

    let Some(phase) = matches.get_one::<Id>("phase") else {
        bail!("Code Generation for ECO32 not yet implemented")
    };

    if phase == "parse" {
        eprintln!("{absyn:#?}");
        return Ok(());
    }

    let table = build_symbol_table(&absyn)?;

    if phase == "tables" {
        eprintln!("{table:#?}");
        return Ok(());
    }

    absyn
        .definitions
        .iter_mut()
        .try_for_each(|def| check_def_global(def, &table))?;

    if phase == "semant" {
        return Ok(());
    }

    if phase == "interpret" {
        let t = table.borrow();
        start_main(&absyn, &t);
        return Ok(());
    }

    let mut address_code = Tac::new(table.clone());
    address_code.code_generation(&absyn);

    if phase == "tac" {
        eprintln!("{address_code}");
        return Ok(());
    }

    let graphs = address_code.proc_table.keys().collect::<Vec<_>>();
    let theme = ColorfulTheme::default();

    let sel_proc = matches
        .get_one::<String>("proc")
        .and_then(|proc_arg| graphs.iter().position(|p| p == &proc_arg));

    let sel_proc = if let Some(sel_proc) = sel_proc {
        sel_proc
    } else {
        Select::with_theme(&theme)
            .with_prompt("Which procedure?")
            .items(&graphs)
            .default(0)
            .interact()?
    };
    let proc_name = graphs[sel_proc];
    let mut graph = BlockGraph::from_tac(address_code.proc_table.get(proc_name).unwrap());

    if phase == "interprettac" {
        let proc_graphs = address_code
            .proc_table
            .into_iter()
            .map(|(proc_name, quads)| (proc_name, BlockGraph::from_tac(&quads)))
            .collect::<HashMap<_, _>>();
        let t = table.borrow();
        eval_tac(&proc_graphs, &t);
        return Ok(());
    }

    if let Some(optis) = matches.get_many::<String>("optis") {
        graph.run_optimizations(optis, &table, proc_name, matches, &theme)?;
    }

    if phase == "dot" {
        graph.show_dot(proc_name, matches, &theme)?;
        return Ok(());
    }

    unreachable!()
}

impl BlockGraph {
    fn run_optimizations<'a>(
        &mut self,
        optis: impl Iterator<Item = &'a String>,
        symbol_table: &Rc<RefCell<SymbolTable>>,
        proc_name: &str,
        matches: &clap::ArgMatches,
        theme: &impl dialoguer::theme::Theme,
    ) -> anyhow::Result<()> {
        for opti in optis {
            let proc_def = symbol_table.borrow().lookup(proc_name);
            let Some(Entry::ProcedureEntry(proc_def)) = proc_def else {
                unreachable!()
            };

            match opti.as_str() {
                "dot" => {
                    println!("{}", ">>> Showing Dot Graph...".green());
                    self.show_dot(proc_name, matches, theme)?;
                }
                "cse" => {
                    eprintln!("{}", ">>> Common Subexpression Elimination".green());
                    let local_table = proc_def.local_table.clone();
                    self.common_subexpression_elimination(&local_table);
                    match symbol_table.borrow_mut().entries.get_mut(proc_name) {
                        Some(Entry::ProcedureEntry(pe)) => pe.local_table = local_table,
                        _ => unreachable!(),
                    }
                }
                "rch" => {
                    eprintln!("{}", ">>> Reaching Definitions:".green());
                    let rch = ReachingDefinitions::run(self, &proc_def.local_table);
                    show_worklist_table(
                        ("Definitions", &rch.defs),
                        (1, 1),
                        ("GEN", &rch.gen_bits),
                        ("PRSV", &rch.prsv),
                        ("RCHin", &rch.rchin),
                        ("RCHout", &rch.rchout),
                        fmt_bitvec,
                    )?;
                }
                "lv" => {
                    eprintln!("{}", ">>> Live Variables:".green());
                    let lv = LiveVariables::run(self, &proc_def.local_table);
                    show_worklist_table(
                        ("Variables", &lv.vars),
                        (1, 1),
                        ("DEF", &lv.def),
                        ("USE", &lv.use_bits),
                        ("LIVin", &lv.livin),
                        ("LIVout", &lv.livout),
                        fmt_bitvec,
                    )?;
                }
                "dead" => {
                    eprintln!("{}", ">>> Dead Code Elimination".green());
                    let lv = LiveVariables::run(self, &proc_def.local_table);
                    self.dead_code_elimination(&lv);
                }
                "gcp" => {
                    eprintln!("{}", ">>> Constant Propagation:".green());
                    let gcp = ConstantPropagation::run(self, &proc_def.local_table);
                    show_worklist_table(
                        ("Variables", &gcp.vars),
                        (5, 14),
                        ("GEN", &gcp.gens),
                        ("PRSV", &gcp.prsv),
                        ("IN", &gcp.r#in),
                        ("OUT", &gcp.out),
                        |v| format!("{v:?}"),
                    )?;
                }
                "cf" => {
                    println!("{}", ">>> Constant Folding (×1)".green());
                    let mut gcp = ConstantPropagation::run(self, &proc_def.local_table);
                    _ = self.constant_folding(&mut gcp, &symbol_table.borrow());
                }
                "cf+" => {
                    println!("{}", ">>> Constant Folding (until stable)".green());
                    let mut gcp = ConstantPropagation::run(self, &proc_def.local_table);
                    let mut iterations = 0;
                    while { self.constant_folding(&mut gcp, &symbol_table.borrow()) }.is_continue()
                    {
                        iterations += 1;
                    }
                    println!("    iterations: {iterations}");
                }
                "scc" => {
                    eprintln!("{}", ">>> Strongly Connected Components:".green());
                    let scc = self.tarjan();
                    eprintln!("{scc:#?}");
                }
                "licm" => {
                    eprintln!("{}", ">>> Loop Invariant Code Motion:".green());
                    self.loop_optimization(&proc_def.local_table);
                }
                "licm+" => {
                    eprintln!("{}", ">>> Loop Invariant Code Motion:".green());
                    while self.loop_optimization(&proc_def.local_table) {}
                }
                _ => panic!("Unknown optimization: {opti}"),
            }
            eprintln!();
        }

        Ok(())
    }

    fn show_dot(
        &self,
        proc_name: &str,
        matches: &clap::ArgMatches,
        theme: &impl dialoguer::theme::Theme,
    ) -> Result<(), anyhow::Error> {
        let mut filename = format!("{proc_name}.dot");
        let outputname = format!("as file: {filename}");
        let outputs = ["print", &outputname, "dot Tx11", "xdot"];

        let output = matches
            .get_one::<String>("dot")
            .and_then(|arg| {
                if Path::new(arg)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("dot"))
                {
                    filename = arg.to_string();
                    Some(1)
                } else {
                    outputs.iter().position(|o| o == arg)
                }
            })
            .map_or_else(
                || {
                    if std::io::stdout().is_terminal() {
                        Select::with_theme(theme)
                            .with_prompt("Which output mode?")
                            .items(&outputs)
                            .default(0)
                            .interact()
                    } else {
                        Ok(0) // always write dot code to stdout if not terminal
                    }
                },
                Ok,
            )?;

        match output {
            0 => {
                println!("{self}");
            }
            1 => {
                let mut file = File::create(filename)?;
                writeln!(file, "{self}")?;
            }
            2 => {
                ShowDot::DotTx11.show(self)?;
            }
            3 => {
                ShowDot::XDot.show(self)?;
            }
            _ => {
                bail!("No valid output given");
            }
        }

        Ok(())
    }
}

fn show_worklist_table<L: Lattice, D: FmtTable>(
    (defs_name, defs): (&str, &[D]),
    (col_width_factor, col_width_factor_colored): (usize, usize),
    (al, av): (&str, &[L]),
    (bl, bv): (&str, &[L]),
    (il, iv): (&str, &[L]),
    (ol, ov): (&str, &[L]),
    f: impl Fn(&L) -> String,
) -> fmt::Result {
    eprintln!("{defs_name}:");
    eprintln!("{}", D::fmt_table(defs)?);
    eprintln!();

    let label_len = [al, bl, il, ol].iter().map(|s| s.len()).max().unwrap();
    let col_width = (defs.len() * col_width_factor).max(label_len);
    eprintln!(
        "{:>5} {:<col_width$} {:<col_width$} {:<col_width$} {:<col_width$}",
        "Block", al, bl, il, ol,
    );
    let col_width = (defs.len() * col_width_factor_colored).max(label_len);
    for (n, (((a, b), i), o)) in av.iter().zip(bv).zip(iv).zip(ov).enumerate() {
        eprintln!(
            "{n:>5} {:<col_width$} {:<col_width$} {:<col_width$} {:<col_width$}",
            f(a),
            f(b),
            f(i),
            f(o),
        );
    }

    Ok(())
}

fn fmt_bitvec(bv: &BitVec) -> String {
    bv.iter()
        .map(|b| b.then_some('1').unwrap_or('0'))
        .collect::<String>()
}

pub trait FmtTable: Sized {
    fn fmt_table(defs: &[Self]) -> Result<String, fmt::Error>;
}
impl<D: Display> FmtTable for D {
    fn fmt_table(defs: &[Self]) -> Result<String, fmt::Error> {
        let mut out = String::new();

        for (i, v) in defs.iter().enumerate() {
            writeln!(out, "{i:>5}: {v}")?;
        }

        Ok(out)
    }
}

enum ShowDot {
    XDot,
    DotTx11,
}
impl ShowDot {
    fn show(&self, graph: &BlockGraph) -> anyhow::Result<process::ExitStatus> {
        let (bin_name, args, test_args) = self.cmd_info();

        process::Command::new(bin_name)
            .args(test_args)
            .output()
            .map(|output| output.status.success())
            .map_err(|_| anyhow!("dot is not installed!".red()))?;

        let mut dot = process::Command::new("dot")
            .args(args)
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = dot.stdin.as_mut() {
            write!(stdin, "{graph}")?;
        }
        Ok(dot.wait()?)
    }

    const fn cmd_info(&self) -> (&str, &[&str], &[&str]) {
        match self {
            Self::XDot => ("xdot", &["-"], &["-h"]),
            Self::DotTx11 => ("dot", &["-Tx11"], &["--version"]),
        }
    }
}
