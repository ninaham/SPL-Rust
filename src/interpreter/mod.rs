pub mod definition_evaluator;
pub mod environment;
pub mod expression_evaluator;
pub mod statement_evaluator;
pub mod tac_interpreter;
pub mod value;

#[cfg(test)]
mod test {
    use std::{
        collections::HashMap,
        fs,
        path::{Path, PathBuf},
    };

    use rstest::rstest;

    use crate::{
        base_blocks::BlockGraph,
        code_gen::Tac,
        interpreter::tac_interpreter::eval_tac,
        parser::parse_everything_else::parse,
        semant::{build_symbol_table::build_symbol_table, check_def_global},
    };

    use super::definition_evaluator::start_main;

    #[rstest]
    fn ast(
        #[files("spl-testfiles/runtime_tests/*.spl")]
        #[exclude("reftest.spl")]
        #[exclude("test8.spl")]
        #[exclude("test9.spl")]
        path: PathBuf,
    ) -> anyhow::Result<()> {
        test_file_ast(&path)
    }

    fn test_file_ast(path: &Path) -> anyhow::Result<()> {
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

    #[test]
    fn queens_ast() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/queens.spl")).unwrap();
    }

    #[test]
    fn queens_tac() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/queens.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "index out of bounds for array length 3: -1")]
    fn runtime_err_ast_8() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/test8.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "index out of bounds for array length 3: 3")]
    fn runtime_err_ast_9() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/test9.spl")).unwrap();
    }

    #[rstest]
    fn tac(
        #[files("spl-testfiles/runtime_tests/*.spl")]
        #[exclude("reftest.spl")]
        #[exclude("test8.spl")]
        #[exclude("test9.spl")]
        path: PathBuf,
    ) -> anyhow::Result<()> {
        test_file_tac(&path)
    }

    fn test_file_tac(path: &Path) -> anyhow::Result<()> {
        let code = fs::read_to_string(path).unwrap();

        let mut absyn = parse(code.leak())?;

        let table = build_symbol_table(&absyn)?;

        absyn
            .definitions
            .iter_mut()
            .try_for_each(|def| check_def_global(def, &table))?;

        let mut tac = Tac::new();
        tac.code_generation(&absyn);

        let proc_graphs = tac
            .proc_table
            .into_iter()
            .map(|(proc_name, quads)| (proc_name, BlockGraph::from_tac(&quads)))
            .collect::<HashMap<_, _>>();

        let t = table.borrow();
        eval_tac(&proc_graphs, &t);

        Ok(())
    }

    #[test]
    #[should_panic(expected = "index out of bounds for array length 3: -1")]
    fn test_runtime_err_tac_8() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/test8.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "index out of bounds for array length 3: 3")]
    fn test_runtime_err_tac_9() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/test9.spl")).unwrap();
    }
}
