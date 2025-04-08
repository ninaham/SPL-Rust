//use parser::parser::alphanumeric;

use clap::Parser;

pub mod absyn;
pub mod cli;
pub mod parser;
pub mod tokens;

fn main() {
    //let (rem, m) = alphanumeric("asd_").unwrap();

    //println!("rem: {}", rem);
    //println!("m: {}", m);
    //
    let _cli = cli::Cli::parse();
    //println!("{:?}", cli);
}
