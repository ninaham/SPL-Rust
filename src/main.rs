use cli::process_matches;

mod absyn;
mod base_blocks;
mod cli;
mod code_gen;
mod optimizations;
mod parser;
mod semant;
mod table;

fn main() -> anyhow::Result<()> {
    process_matches(&cli::load_program_data().get_matches())
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::base_blocks::BlockGraph;
    use crate::code_gen::Tac;
    use crate::parser::parse_everything_else::parse;
    use crate::semant::{build_symbol_table::build_symbol_table, check_def_global};

    #[rstest]
    fn test_all_files(
        #[files("spl-testfiles/runtime_tests/*.spl")]
        #[exclude("reftest.spl")]
        path: PathBuf,
    ) -> anyhow::Result<()> {
        test_file(&path)
    }

    fn test_file(path: &Path) -> anyhow::Result<()> {
        let code = fs::read_to_string(path).unwrap();

        let mut absyn = parse(code.leak())?;

        let table = build_symbol_table(&absyn)?;

        absyn
            .definitions
            .iter_mut()
            .try_for_each(|def| check_def_global(def, table.clone()))?;

        let mut address_code = Tac::new(table.clone());
        address_code.code_generation(&absyn);

        assert!(address_code.proc_table.contains_key("main"));

        for code in address_code.proc_table.values() {
            let mut bg = BlockGraph::from_tac(code);
            bg.common_subexpression_elimination(&table.lock().unwrap());
            bg.to_string();
        }

        Ok(())
    }
}
