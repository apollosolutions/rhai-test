use colored::*;
use rhai::FnPtr;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Test {
    pub name: String,
    pub test_function: FnPtr,
    pub file_path: String,
}

impl Test {
    fn new(name: String, test_function: FnPtr, file_path: String) -> Self {
        Self {
            name,
            test_function,
            file_path,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestSuite {
    #[allow(dead_code)]
    file_path: String,
    is_passed: bool,
}

impl TestSuite {
    fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
            is_passed: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestContainer {
    pub tests: Vec<Test>,
    pub test_suites: HashMap<String, TestSuite>,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub expect_results: Vec<Result<(), String>>,
}

impl TestContainer {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            test_suites: HashMap::new(),
            passed_tests: 0,
            failed_tests: 0,
            expect_results: Vec::new(),
        }
    }

    pub fn add_suite(&mut self, file_path: &str) {
        if !self.test_suites.contains_key(file_path) {
            self.test_suites
                .insert(file_path.to_string(), TestSuite::new(&file_path));
        }
    }

    pub fn add_test(&mut self, name: &str, func: FnPtr, file_path: &str) {
        self.tests
            .push(Test::new(name.to_string(), func, file_path.to_string()));
    }

    pub fn get_tests(&self) -> &Vec<Test> {
        &self.tests
    }

    pub fn has_failed_suites(&self) -> bool {
        self.test_suites
            .iter()
            .find(|(_, suite)| !suite.is_passed)
            .is_some()
    }

    pub fn fail_suite(&mut self, path: &str) {
        self.test_suites
            .entry(path.to_string())
            .and_modify(|test_suite| {
                test_suite.is_passed = false;
            });
    }

    pub fn print_results(&mut self) {
        let count_passed_test_suites = self
            .test_suites
            .values()
            .filter(|suite| suite.is_passed)
            .count();
        let count_failed_test_suites = self
            .test_suites
            .values()
            .filter(|suite| !suite.is_passed)
            .count();

        println!("\r\n");
        if count_failed_test_suites > 0 {
            println!(
                "Test Suites: {} {}, {} {}, {} total",
                count_passed_test_suites.to_string().green(),
                "passed".green(),
                count_failed_test_suites.to_string().red(),
                "failed".red(),
                self.test_suites.len()
            );
        } else {
            println!(
                "Test Suites: {} {}, {} total",
                count_passed_test_suites.to_string().green(),
                "passed".green(),
                self.test_suites.len()
            );
        }

        if self.failed_tests > 0 {
            println!(
                "Tests:       {} {}, {} {}, {} total",
                self.passed_tests.to_string().green(),
                "passed".green(),
                self.failed_tests.to_string().red(),
                "failed".red(),
                self.tests.len()
            );
        } else {
            println!(
                "Tests:       {} {}, {} total",
                self.passed_tests.to_string().green(),
                "passed".green(),
                self.tests.len()
            );
        }
    }

    pub fn add_expect_result(&mut self, result: Result<(), String>) {
        self.expect_results.push(result);
    }

    pub fn clear_expect_results(&mut self) {
        self.expect_results = Vec::new();
    }
}
