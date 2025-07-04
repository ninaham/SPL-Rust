use cli::process_matches;

mod absyn; // Abstract Syntax Tree structures
mod base_blocks; // Control flow graph and basic block handling
mod cli; // CLI parsing and argument handling
mod code_gen; // Code generation (e.g. TAC)
mod interpreter;
mod optimizations; // Compiler optimizations
mod parser; // SPL parser implementation
mod semant; // Semantic checks and symbol table generation
mod spl_builtins;
mod table; // Symbol table and entry types

fn main() -> anyhow::Result<()> {
    // Entry point: parse CLI arguments and start processing
    process_matches(&cli::load_program_data().get_matches())
}

#[cfg(test)]
mod test {
    use rstest::rstest;
    use std::fs;
    use std::path::{Path, PathBuf};

    // Import necessary modules for testing the full compilation pipeline
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
    fn optimizations(
        #[files("spl-testfiles/optimizations/*.spl")] path: PathBuf,
    ) -> anyhow::Result<()> {
        file(&path)
    }

    #[rstest]
    fn runtime_tests(
        #[files("spl-testfiles/runtime_tests/*.spl")]
        #[exclude("reftest.spl")]
        path: PathBuf,
    ) -> anyhow::Result<()> {
        file(&path)
    }

    #[rstest]
    fn syntax_errors(#[files("spl-testfiles/syntax_errors/*.spl")] path: PathBuf) {
        let code = fs::read_to_string(path).unwrap();
        // Parsing should fail (on purpose)
        parse(&code).expect_err("Parsing should fail");
    }

    fn file(path: &Path) -> anyhow::Result<()> {
        let code = fs::read_to_string(path).unwrap();

        // Parse into abstract syntax tree
        let mut absyn = parse(code.leak())?;

        // Build the global symbol table
        let table = build_symbol_table(&absyn)?;

        // Perform semantic checks on all definitions
        absyn
            .definitions
            .iter_mut()
            .try_for_each(|def| check_def_global(def, &table))?;

        let mut address_code = Tac::new(table.clone());
        address_code.code_generation(&absyn);

        // Ensure that a "main" function exists
        assert!(address_code.proc_table.contains_key("main"));

        // Process each procedure independently
        for (proc_name, code) in &address_code.proc_table {
            let Some(Entry::ProcedureEntry(proc_entry)) = table.borrow().lookup(proc_name) else {
                unreachable!()
            };
            let mut bg = BlockGraph::from_tac(code);

            bg.common_subexpression_elimination(&proc_entry.local_table);

            match table.borrow_mut().entries.get_mut(proc_name) {
                Some(Entry::ProcedureEntry(pe)) => pe.local_table = proc_entry.local_table,
                _ => unreachable!(),
            }
            let Some(Entry::ProcedureEntry(proc_entry)) = table.borrow().lookup(proc_name) else {
                unreachable!()
            };

            let mut bg = BlockGraph::from_tac(code);

            bg.common_subexpression_elimination(&table.borrow());

            match table.borrow_mut().entries.get_mut(proc_name) {
                Some(Entry::ProcedureEntry(pe)) => pe.local_table = proc_entry.local_table,
                _ => unreachable!(),
            }
            let Some(Entry::ProcedureEntry(proc_entry)) = table.borrow().lookup(proc_name) else {
                unreachable!()
            };
            let local_table = &proc_entry.local_table;

            // Compute Reaching Definitions (used for many other analyses)
            ReachingDefinitions::run(&bg, local_table);

            // Run liveness analysis to find unused variables
            let live_variables = LiveVariables::run(&bg, local_table);

            // Eliminate dead code based on liveness information
            bg.dead_code_elimination(&live_variables);

            // Run constant propagation analysis
            let mut const_prop = ConstantPropagation::run(&bg, local_table);
            while { bg.constant_folding(&mut const_prop, &table.borrow()) }.is_continue() {}

            // (Optional) Debug print or to trigger formatting
            bg.to_string();
        }

        Ok(())
    }
}
