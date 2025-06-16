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
    use crate::optimizations::constant_propagation::ConstantPropagation;
    use crate::optimizations::live_variables::LiveVariables;
    use crate::optimizations::reaching_expressions::ReachingDefinitions;
    use crate::optimizations::worklist::Worklist;
    use crate::parser::parse_everything_else::parse;
    use crate::semant::{build_symbol_table::build_symbol_table, check_def_global};
    use crate::table::entry::Entry;

    #[rstest]
    fn test_all_files(
        #[files("spl-testfiles/runtime_tests/*.spl")]
        #[exclude("reftest.spl")]
        path: PathBuf,
    ) -> anyhow::Result<()> {
        test_file(&path)
    }

    #[rstest]
    fn test_syntax_errors(#[files("spl-testfiles/syntax_errors/*.spl")] path: PathBuf) {
        let code = fs::read_to_string(path).unwrap();
        parse(&code).expect_err("Parsing should fail");
    }

    fn test_file(path: &Path) -> anyhow::Result<()> {
        let code = fs::read_to_string(path).unwrap();

        let mut absyn = parse(code.leak())?;

        let table = build_symbol_table(&absyn)?;

        absyn
            .definitions
            .iter_mut()
            .try_for_each(|def| check_def_global(def, &table))?;

        let mut address_code = Tac::new(table.clone());
        address_code.code_generation(&absyn);

        assert!(address_code.proc_table.contains_key("main"));

        for (proc_name, code) in &address_code.proc_table {
            let Some(Entry::ProcedureEntry(proc_entry)) = table.lock().unwrap().lookup(proc_name)
            else {
                unreachable!()
            };
            let local_table = &proc_entry.local_table;
            let mut bg = BlockGraph::from_tac(code);

            bg.common_subexpression_elimination(&table.lock().unwrap());

            ReachingDefinitions::run(&mut bg, local_table);

            let live_variables = LiveVariables::run(&mut bg, local_table);
            bg.dead_code_elimination(&live_variables);
            ConstantPropagation::run(&mut bg, local_table);

            bg.to_string();
        }

        Ok(())
    }
}
