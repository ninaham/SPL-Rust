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
        #[exclude("drawTest.spl")] // requires graphics
        #[exclude("gol.spl")] // requires user input
        #[exclude("lambda.spl")] // interactive
        #[exclude("sierpinski.spl")] // requires graphics
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
    #[should_panic(expected = "index out of bounds for array length 3: -1")]
    fn ast_runtime_err_8() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/test8.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "index out of bounds for array length 3: 3")]
    fn ast_runtime_err_9() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/test9.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "not implemented: SPL-builtin `clearAll()`")]
    fn ast_unimplemented_drawtest() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/drawTest.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "not implemented: SPL-builtin `clearAll()`")]
    fn ast_unimplemented_sierpinski() {
        test_file_ast(Path::new("spl-testfiles/runtime_tests/sierpinski.spl")).unwrap();
    }

    #[rstest]
    fn tac(
        #[files("spl-testfiles/runtime_tests/*.spl")]
        #[exclude("reftest.spl")]
        #[exclude("test8.spl")]
        #[exclude("test9.spl")]
        #[exclude("acker.spl")] // TODO: fatal runtime error: stack overflow, aborting
        #[exclude("drawTest.spl")] // requires graphics
        #[exclude("gol.spl")] // requires user input
        #[exclude("lambda.spl")] // interactive
        #[exclude("sierpinski.spl")] // requires graphics
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

        let mut tac = Tac::new(table.clone());
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
    fn tac_runtime_err_8() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/test8.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "index out of bounds for array length 3: 3")]
    fn tac_runtime_err_9() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/test9.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "not implemented: SPL-builtin `clearAll()`")]
    fn tac_unimplemented_drawtest() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/drawTest.spl")).unwrap();
    }

    #[test]
    #[should_panic(expected = "not implemented: SPL-builtin `clearAll()`")]
    fn tac_unimplemented_sierpinski() {
        test_file_tac(Path::new("spl-testfiles/runtime_tests/sierpinski.spl")).unwrap();
    }
}
