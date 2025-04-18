//use parser::parser::alphanumeric;

use cli::process_matches;

pub mod absyn;
pub mod cli;
pub mod code_gen;
pub mod parser;
pub mod semant;
pub mod table;

fn main() -> anyhow::Result<()> {
    process_matches(&cli::load_program_data().get_matches())
}
