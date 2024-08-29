use colored::*;
use rhai::{Engine, EvalAltResult, AST};

use crate::test_container::Test;

pub struct TestSuiteResult {
    pub passed_tests: i32,
    pub failed_tests: i32,
}

impl TestSuiteResult {
    pub fn new() -> Self {
        Self {
            passed_tests: 0,
            failed_tests: 0,
        }
    }
}

pub struct TestRunner {}

impl TestRunner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_tests(
        &self,
        engine: &Engine,
        ast: &AST,
        path: &str,
        tests: &Vec<Test>,
    ) -> TestSuiteResult {
        let mut test_run_result = TestSuiteResult::new();

        for test in tests {
            if test.file_path == path {
                match test
                    .test_function
                    .call::<Result<(), String>>(engine, ast, ())
                {
                    Ok(result) => match result {
                        Ok(()) => {
                            println!("\t{} {}", " PASS ".white().on_green().bold(), test.name);
                            test_run_result.passed_tests += 1;
                        }
                        Err(error) => {
                            println!("\t{} {}", " FAIL ".black().on_red().bold(), test.name);
                            println!("\t\t{} {}", "Reason:".red(), error.to_string().red());
                            test_run_result.failed_tests += 1;
                        }
                    },
                    Err(error) => {
                        println!("\t{} {}", " FAIL ".black().on_red().bold(), test.name);
                        println!("\t\t{} {}", "Reason:".red(), error.to_string().red());

                        match *error {
                            EvalAltResult::ErrorMismatchOutputType(_, _, _) => {
                                println!(
                                    "{}",
                                    "\t\tHint: Make sure your test ends with an expect function."
                                        .green()
                                )
                            }
                            _ => (),
                        }
                        test_run_result.failed_tests += 1;
                    }
                }
            }
        }

        return test_run_result;
    }
}
