pub mod definition_evaluator;
pub mod environment;
pub mod expression_evaluator;
pub mod statement_evaluator;
pub mod value;

#[cfg(test)]
mod test {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use rstest::rstest;

    use crate::{
        parser::parse_everything_else::parse,
        semant::{build_symbol_table::build_symbol_table, check_def_global},
    };

    use super::definition_evaluator::start_main;

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
            .try_for_each(|def| check_def_global(def, &table))?;

        let t = table.borrow();
        start_main(&absyn, &t);

        Ok(())
    }
}
