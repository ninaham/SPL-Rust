use clap::{ArgGroup, Parser};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLI_INPUT: Cli = Cli::parse();
}

#[derive(Parser, Debug)]
#[command(
    name = "spl compiler",
    author,
    version,
    about,
    group(
        ArgGroup::new("phase")
            .required(true)
            .args([
                "tokens", "parse", "absyn", "tables", "semant", "vars", "tac"
            ])
    )
)]

pub struct Cli {
    /// Phase 1: Scans for tokens and prints them.
    #[arg(long)]
    tokens: bool,

    /// Phase 2: Parses the stream of tokens to check for syntax errors.
    #[arg(long)]
    parse: bool,

    /// Phase 3: Creates an abstract syntax tree from the input tokens and prints it.
    #[arg(long)]
    absyn: bool,

    /// Phase 4a: Builds a symbol table and prints its entries.
    #[arg(long)]
    tables: bool,

    /// Phase 4b: Performs the semantic analysis.
    #[arg(long)]
    semant: bool,

    /// Phase 5: Allocates memory space for variables and prints the amount of allocated memory.
    #[arg(long)]
    vars: bool,

    /// Show the TAC
    #[arg(long)]
    tac: bool,
}

#[derive(Debug)]
pub enum Phase {
    Tokens,
    Parse,
    Absyn,
    Tables,
    Semant,
    Vars,
    Tac,
}

pub fn parse_phase(cli: &Cli) -> Phase {
    match () {
        _ if cli.tokens => Phase::Tokens,
        _ if cli.parse => Phase::Parse,
        _ if cli.absyn => Phase::Absyn,
        _ if cli.tables => Phase::Tables,
        _ if cli.semant => Phase::Semant,
        _ if cli.vars => Phase::Vars,
        _ if cli.tac => Phase::Tac,
        _ => unreachable!("clap garantiert, dass genau eine Option gewählt wurde"),
    }
}

pub fn handle_phase(phase: Phase) {
    match phase {
        Phase::Tokens => println!("Phase 1: Scans for tokens and prints them."),
        Phase::Parse => {
            println!("Phase 2: Parses the stream of tokens to check for syntax errors.")
        }
        Phase::Absyn => println!(
            "Phase 3: Creates an abstract syntax tree from the input tokens and prints it."
        ),
        Phase::Tables => println!("Phase 4a: Builds a symbol table and prints its entries."),
        Phase::Semant => println!("Phase 4b: Performs the semantic analysis."),
        Phase::Vars => println!(
            "Phase 5: Allocates memory space for variables and prints the amount of allocated memory."
        ),
        Phase::Tac => println!("Show the TAC"),
    }
}
