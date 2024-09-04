use crate::coverage_reporting::test_coverage_container::TestCoverageContainer;
use crate::engine::engine::create_engine;
use crate::Config;
use colored::Colorize;
use regex::Regex;
use rhai::{Dynamic, EvalAltResult, FnPtr, ImmutableString, Map, Position, AST};
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use super::error_handling::{get_inner_most_error, get_stack_trace, get_stack_trace_output};

#[derive(Debug, Clone)]
pub enum ExpectedValue {
    String(String),
    Bool(bool),
    Function(FnPtr),
}

impl ExpectedValue {
    pub fn from_dynamic(dynamic: &Dynamic) -> Result<Self, Box<EvalAltResult>> {
        if let Some(s) = dynamic.clone().try_cast::<ImmutableString>() {
            Ok(ExpectedValue::String(s.to_string()))
        } else if let Some(b) = dynamic.clone().try_cast::<bool>() {
            Ok(ExpectedValue::Bool(b))
        } else if let Some(f) = dynamic.clone().try_cast::<FnPtr>() {
            Ok(ExpectedValue::Function(f))
        } else {
            Err("Unsupported type".into())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expector {
    pub value: ExpectedValue,
    pub negative: bool,
    ast: Option<Arc<Mutex<Option<AST>>>>,
    test_coverage_container: Option<Arc<Mutex<TestCoverageContainer>>>,
    config: Option<Arc<Mutex<Config>>>,
}

impl Expector {
    pub fn new(value: Dynamic) -> Self {
        Self {
            value: ExpectedValue::from_dynamic(&value).unwrap(),
            negative: false,
            ast: None,
            test_coverage_container: None,
            config: None,
        }
    }

    pub fn attach(
        &mut self,
        ast: Arc<Mutex<Option<AST>>>,
        test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
        config: Arc<Mutex<Config>>,
    ) {
        self.ast = Some(ast);
        self.test_coverage_container = Some(test_coverage_container);
        self.config = Some(config);
    }

    pub fn not(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn to_be(&mut self, expected: Dynamic) -> Result<(), String> {
        let condition = match (
            &self.value,
            &ExpectedValue::from_dynamic(&expected).unwrap(),
        ) {
            (ExpectedValue::String(value), ExpectedValue::String(expected_value)) => {
                value == expected_value
            }
            (ExpectedValue::Bool(value), ExpectedValue::Bool(expected_value)) => {
                value == expected_value
            }
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        if !condition && !self.negative {
            let error = format!(
                "Expected value to be {:?} but instead got {:?}",
                expected, self.value
            );

            Err(error)
        } else if condition && self.negative {
            let error = format!(
                "Expected value {:?} to not be {:?} but it was",
                self.value, expected
            );

            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn to_match(&mut self, pattern: &str) -> Result<(), String> {
        let regex = Regex::new(pattern).unwrap();

        let condition = match &self.value {
            ExpectedValue::String(value) => regex.is_match(value),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        if !condition && !self.negative {
            let error = format!(
                "Expected value {:?} to match pattern {:?} but it did not",
                self.value, pattern
            );

            Err(error)
        } else if condition && self.negative {
            let error = format!(
                "Expected value {:?} to not match pattern {:?} but it did",
                self.value, pattern
            );

            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn to_throw_status_and_message(
        &mut self,
        status_code_to_match: i64,
        message_to_match: &str,
    ) -> Result<(), String> {
        let check1 = self.to_throw_status(status_code_to_match);
        let check2 = self.to_throw_message(message_to_match);

        if check1.is_err() {
            return check1;
        } else if check2.is_err() {
            return check2;
        } else {
            Ok(())
        }
    }

    // TODO: Refactor these "to throw" methods to be less repetitive if possible
    pub fn to_throw_status(&mut self, status_code_to_match: i64) -> Result<(), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();
        let test_coverage_container = self.test_coverage_container.clone().unwrap();
        let config = self.config.clone().unwrap();

        let engine = create_engine(test_coverage_container, config);

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        let mut status_code = String::new();

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err);
            status_code = stack_trace.last().unwrap().status_code.clone();
            let inner_most_error = get_inner_most_error(err);

            if !matches!(**inner_most_error, rhai::EvalAltResult::ErrorRuntime(..)) {
                return Err(get_stack_trace_output(
                    "Unexpected error ocurred when running tests.".to_string(),
                    &stack_trace,
                ));
            }
        }

        let condition = result.is_err();
        let condition2 = status_code == status_code_to_match.to_string();

        if !condition && !self.negative {
            let error = format!("Expected function to throw but it did not");

            Err(error)
        } else if condition && self.negative {
            let error = format!("Expected function to not throw but it did");

            Err(error)
        } else if condition && !condition2 {
            Err(format!(
                "Expected function to throw error with status '{}' but instead received '{}'",
                status_code_to_match, status_code
            ))
        } else {
            Ok(())
        }
    }

    pub fn to_throw_message(&mut self, message_to_match: &str) -> Result<(), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();
        let test_coverage_container = self.test_coverage_container.clone().unwrap();
        let config = self.config.clone().unwrap();

        let engine = create_engine(test_coverage_container, config);

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        let mut message = String::new();

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err);
            message = stack_trace.last().unwrap().message.clone();
            let inner_most_error = get_inner_most_error(err);

            if !matches!(**inner_most_error, rhai::EvalAltResult::ErrorRuntime(..)) {
                return Err(get_stack_trace_output(
                    "Unexpected error ocurred when running tests.".to_string(),
                    &stack_trace,
                ));
            }
        }

        let condition = result.is_err();
        let condition2 = message == message_to_match;
        let condition3 = {
            let regex = Regex::new(message_to_match);
            let condition3_result;

            match regex {
                Ok(regex) => {
                    condition3_result = regex.is_match(&message);
                }
                Err(_) => {
                    condition3_result = false;
                }
            }

            condition3_result
        };

        if !condition && !self.negative {
            let error = format!("Expected function to throw but it did not");

            Err(error)
        } else if condition && self.negative {
            let error = format!("Expected function to not throw but it did");

            Err(error)
        } else if condition && (!condition2 && !condition3) {
            Err(format!(
                "Expected function to throw error with message '{}' but instead received '{}'",
                message_to_match, message
            ))
        } else {
            Ok(())
        }
    }

    pub fn to_throw(&mut self) -> Result<(), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();
        let test_coverage_container = self.test_coverage_container.clone().unwrap();
        let config = self.config.clone().unwrap();

        let engine = create_engine(test_coverage_container, config);

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err);
            let inner_most_error = get_inner_most_error(err);

            if !matches!(**inner_most_error, rhai::EvalAltResult::ErrorRuntime(..)) {
                return Err(get_stack_trace_output(
                    "Unexpected error ocurred when running tests.".to_string(),
                    &stack_trace,
                ));
            }
        }

        let condition = result.is_err();

        if !condition && !self.negative {
            let error = format!("Expected function to throw but it did not");

            Err(error)
        } else if condition && self.negative {
            let error = format!("Expected function to not throw but it did");

            Err(error)
        } else {
            Ok(())
        }
    }
}
