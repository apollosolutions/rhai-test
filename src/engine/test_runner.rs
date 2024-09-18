use super::{logging_container::LoggingContainer, test_container::TestContainer};
use crate::engine::test_container::Test;
use colored::*;
use rhai::{Engine, EvalAltResult, AST};
use std::sync::{Arc, Mutex};

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
        logging_container: Arc<Mutex<LoggingContainer>>,
        test_container: Arc<Mutex<TestContainer>>,
    ) -> TestSuiteResult {
        let mut test_run_result = TestSuiteResult::new();
        let mut test_results = Vec::<TestResult>::new();
        let mut all_passing = true;

        for test in tests {
            if test.file_path == path {
                // Execute the test's function body
                match test.test_function.call::<()>(engine, ast, ()) {
                    Ok(_) => {
                        // Get the results registered by the expect statements and see if we have any errors
                        let locked_container = test_container.lock().unwrap();
                        let first_error = locked_container
                            .expect_results
                            .iter()
                            .find_map(|r| r.as_ref().err());

                        // If we have any errors, test failed, otherwise, passed
                        if first_error.is_some() {
                            test_results.push(TestResult::new(
                                test.name.clone(),
                                false,
                                first_error.unwrap().to_string(),
                            ));
                            test_run_result.failed_tests += 1;
                            all_passing = false;
                        } else {
                            test_results.push(TestResult::new(
                                test.name.clone(),
                                true,
                                "".to_string(),
                            ));
                            test_run_result.passed_tests += 1;
                        }
                    }
                    Err(error) => {
                        let mut reason = error.to_string();

                        match *error {
                            EvalAltResult::ErrorMismatchOutputType(_, _, _) => {
                                let hint =
                                    format!( "{}",
                                "\n\t\tHint: Make sure your test ends with an expect function."
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
                // We need to reset some of our containers after each test since these track things on a test-by-test basis and expector functions don't know which test they are running in
                logging_container.lock().unwrap().reset();
                test_container.lock().unwrap().clear_expect_results();
            }
        }

        // Did the suite pass?
        if all_passing {
            println!("{} {}", " PASS ".white().on_green().bold(), path);
        } else {
            println!("{} {}", " FAIL ".white().on_red().bold(), path);
        }

        // Output the result of each individual test
        test_results.iter().for_each(|test_result| {
            if test_result.is_passed {
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
