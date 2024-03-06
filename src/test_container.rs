use std::collections::HashMap;

use rhai::{Engine, EvalAltResult, FnPtr, AST};
use colored::*;

pub struct Test {
    name: String,
    test_function: FnPtr,
    file_path: String
}

impl Test {
    fn new(name: String, test_function: FnPtr, file_path: String) -> Self {
        Self {
            name,
            test_function,
            file_path
        }
    }
}

pub struct TestSuite {
    #[allow(dead_code)]
    file_path: String,
    is_passed: bool
}

impl TestSuite {
    fn new(file_path: &str) -> Self{
        Self {
            file_path: file_path.to_string(),
            is_passed: true
        }
    }
}

pub struct TestContainer {
    pub tests: Vec<Test>,
    pub test_suites: HashMap<String, TestSuite>,
    pub passed_tests: i32,
    pub failed_tests: i32,
}

impl TestContainer {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            test_suites: HashMap::new(),
            passed_tests: 0,
            failed_tests: 0
        }
    }

    pub fn add_test(&mut self, name: &str, func: FnPtr, file_path: &str) {
        self.tests.push(Test::new(name.to_string(), func, file_path.to_string()));
        if !self.test_suites.contains_key(file_path){
            self.test_suites.insert(file_path.to_string(), TestSuite::new(&file_path));
        }
    }

    pub fn run_tests(&mut self, engine: &Engine, ast: &AST, path: &str){
        for test in &self.tests {
            if test.file_path == path {
                match test.test_function.call::<Result<(), String>>(engine, ast, ()) {
                    Ok(result) => {
                        match result {
                            Ok(()) => {
                                println!("\t{} {}", " PASS ".white().on_green().bold(), test.name);
                                self.passed_tests += 1;
                            }
                            Err(error) => {
                                println!("\t{} {}", " FAIL ".black().on_red().bold(), test.name);
                                println!("\t\t{} {}", "Reason:".red(), error.to_string().red());
                                self.failed_tests += 1;
                                self.test_suites
                                .entry(path.to_string())
                                .and_modify(|test_suite| {
                                    test_suite.is_passed = false;
                                });
                            }
                        }
                    }
                    Err(error) => {
                        println!("\t{} {}", " FAIL ".black().on_red().bold(), test.name);
                        println!("\t\t{} {}", "Reason:".red(), error.to_string().red());

                        match *error {
                            EvalAltResult::ErrorMismatchOutputType(_,_,_) => {
                                println!("{}", "\t\tHint: Make sure your test ends with an expect function.".green())
                            }
                            _ => ()
                        }
                        self.failed_tests += 1;
                        self.test_suites
                        .entry(path.to_string())
                        .and_modify(|test_suite| {
                            test_suite.is_passed = false;
                        });
                    }
                }
            }
        }
    }

    pub fn print_results(&mut self){
        let count_passed_test_suites = self.test_suites.values().filter(|suite| suite.is_passed).count();
        let count_failed_test_suites = self.test_suites.values().filter(|suite| !suite.is_passed).count();

        println!("\r\n");
        if count_failed_test_suites > 0 {
            println!("Test Suites: {} {}, {} {}, {} total", count_passed_test_suites.to_string().green(), "passed".green(), count_failed_test_suites.to_string().red(), "failed".red(), self.test_suites.len());
        } else {
            println!("Test Suites: {} {}, {} total", count_passed_test_suites.to_string().green(), "passed".green(), self.test_suites.len());
        }

        if self.failed_tests > 0 {
            println!("Tests:       {} {}, {} {}, {} total", self.passed_tests.to_string().green(), "passed".green(), self.failed_tests.to_string().red(), "failed".red(), self.tests.len());
        } else {
            println!("Tests:       {} {}, {} total", self.passed_tests.to_string().green(), "passed".green(), self.tests.len());
        }


        
    }
}