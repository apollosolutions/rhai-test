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

pub struct TestResult {
    pub name: String,
    pub is_passed: bool,
    pub reason: String,
}

impl TestResult {
    pub fn new(name: String, is_passed: bool, reason: String) -> Self {
        Self {
            name,
            is_passed,
            reason,
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
        let mut test_results = Vec::<TestResult>::new();
        let mut all_passing = true;

        for test in tests {
            if test.file_path == path {
                match test
                    .test_function
                    .call::<Result<(), String>>(engine, ast, ())
                {
                    Ok(result) => match result {
                        Ok(()) => {
                            test_results.push(TestResult::new(
                                test.name.clone(),
                                true,
                                "".to_string(),
                            ));
                            test_run_result.passed_tests += 1;
                        }
                        Err(error) => {
                            test_results.push(TestResult::new(
                                test.name.clone(),
                                false,
                                error.to_string(),
                            ));
                            test_run_result.failed_tests += 1;
                            all_passing = false;
                        }
                    },
                    Err(error) => {
                        let mut reason = error.to_string();

                        match *error {
                            EvalAltResult::ErrorMismatchOutputType(_, _, _) => {
                                let hint =
                                    format!( "{}",
                                "\t\tHint: Make sure your test ends with an expect function."
                                    .green());
                                reason.push_str(&hint);
                            }
                            _ => (),
                        }

                        test_results.push(TestResult::new(test.name.clone(), false, reason));

                        test_run_result.failed_tests += 1;
                        all_passing = false;
                    }
                }
            }
        }

        if (all_passing) {
            println!("{} {}", " PASS ".white().on_green().bold(), path);
        } else {
            println!("{} {}", " FAIL ".white().on_red().bold(), path);
        }

        test_results.iter().for_each(|test_result| {
            if (test_result.is_passed) {
                println!("\t{} {}", "✓".green().bold(), test_result.name);
            } else {
                println!(
                    "\t{} {}\n\t\t{}",
                    "✗".red().bold(),
                    test_result.name,
                    test_result.reason.red()
                );
            }
        });

        return test_run_result;
    }
}
