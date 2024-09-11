use crate::coverage_reporting::test_coverage_container::TestCoverageContainer;
use crate::engine::engine::create_engine;
use crate::Config;
use regex::Regex;
use rhai::{Dynamic, EvalAltResult, FnPtr, ImmutableString, Module, AST};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use super::{
    error_handling::{get_inner_most_error, get_stack_trace, get_stack_trace_output},
    logging_container::{LogLevel, LoggingContainer},
};

#[derive(Debug, Clone)]
pub enum ExpectedValue {
    String(String),
    Bool(bool),
    Function(FnPtr),
    Error(String),
    LogLevel(LogLevel),
}

impl ExpectedValue {
    // TODO: Support more types, make sure to add to the equal check below
    pub fn from_dynamic(dynamic: &Dynamic) -> Result<Self, String> {
        if let Some(s) = dynamic.clone().try_cast::<ImmutableString>() {
            Ok(ExpectedValue::String(s.to_string()))
        } else if let Some(b) = dynamic.clone().try_cast::<bool>() {
            Ok(ExpectedValue::Bool(b))
        } else if let Some(f) = dynamic.clone().try_cast::<FnPtr>() {
            Ok(ExpectedValue::Function(f))
        } else if let Some(l) = dynamic.clone().try_cast::<LogLevel>() {
            Ok(ExpectedValue::LogLevel(l))
        } else {
            Err(format!(
                "Unsupported type provided to expect() or it's child functions: {}",
                dynamic.type_name()
            )
            .into())
        }
    }
}

impl PartialEq for ExpectedValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpectedValue::String(s1), ExpectedValue::String(s2)) => s1 == s2,
            (ExpectedValue::Bool(b1), ExpectedValue::Bool(b2)) => b1 == b2,
            (ExpectedValue::Function(f1), ExpectedValue::Function(f2)) => {
                f1.to_string() == f2.to_string()
            }
            _ => false,
        }
    }
}

// Don't remove this, this is required!
impl Eq for ExpectedValue {}

#[derive(Debug, Clone)]
pub struct Expector {
    pub value: ExpectedValue,
    pub negative: bool,
    ast: Option<Arc<Mutex<Option<AST>>>>,
    test_coverage_container: Option<Arc<Mutex<TestCoverageContainer>>>,
    config: Option<Arc<Mutex<Config>>>,
    module_cache: Option<Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>>,
    logging_container: Option<Arc<Mutex<LoggingContainer>>>,
}

impl Expector {
    pub fn new(value: Dynamic) -> Self {
        let value_from_dynamic = match ExpectedValue::from_dynamic(&value) {
            Ok(val) => val,
            Err(message) => ExpectedValue::Error(message),
        };

        Self {
            value: value_from_dynamic,
            negative: false,
            ast: None,
            test_coverage_container: None,
            config: None,
            module_cache: None,
            logging_container: None,
        }
    }

    pub fn attach(
        &mut self,
        ast: Arc<Mutex<Option<AST>>>,
        test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
        config: Arc<Mutex<Config>>,
        module_cache: Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>,
        logging_container: Arc<Mutex<LoggingContainer>>,
    ) {
        self.ast = Some(ast);
        self.test_coverage_container = Some(test_coverage_container);
        self.config = Some(config);
        self.module_cache = Some(module_cache);
        self.logging_container = Some(logging_container);
    }

    pub fn not(mut self) -> Self {
        self.negative = true;
        self
    }

    pub fn to_be(&mut self, expected: Dynamic) -> Result<(), String> {
        if let ExpectedValue::Error(err_msg) = &self.value {
            return Err(err_msg.clone());
        }

        let condition = &self.value == &ExpectedValue::from_dynamic(&expected)?;

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
        if let ExpectedValue::Error(err_msg) = &self.value {
            return Err(err_msg.clone());
        }

        let regex = Regex::new(pattern).unwrap();

        let condition = match &self.value {
            ExpectedValue::String(value) => regex.is_match(value),
            _ => return Err("Expected value passed to expect() to be a string".to_string()),
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

    pub fn to_throw_status(&mut self, status_code_to_match: i64) -> Result<(), String> {
        if let ExpectedValue::Error(err_msg) = &self.value {
            return Err(err_msg.clone());
        }

        let (result, _, status_code) = &self.run_throw_function()?;

        let condition = result.is_err();
        let condition2 = status_code.clone() == status_code_to_match.to_string();

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
        if let ExpectedValue::Error(err_msg) = &self.value {
            return Err(err_msg.clone());
        }

        let (result, message, ..) = &self.run_throw_function()?;

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
        if let ExpectedValue::Error(err_msg) = &self.value {
            return Err(err_msg.clone());
        }

        let (result, ..) = &self.run_throw_function()?;

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

    fn run_throw_function(
        &mut self,
    ) -> Result<(Result<(), Box<EvalAltResult>>, String, String), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();
        let test_coverage_container = self.test_coverage_container.clone().unwrap();
        let config = self.config.clone().unwrap();
        let module_cache = self.module_cache.clone().unwrap();
        let logging_container = self.logging_container.clone().unwrap();

        let engine = create_engine(
            test_coverage_container,
            config,
            module_cache,
            logging_container,
        );

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Expected value passed to expect() to be a function".to_string()),
        };

        let mut message = String::new();
        let mut status_code = String::new();

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err, None);
            message = stack_trace.last().unwrap().message.clone();
            status_code = stack_trace.last().unwrap().status_code.clone();
            let inner_most_error = get_inner_most_error(err);

            if !matches!(**inner_most_error, rhai::EvalAltResult::ErrorRuntime(..)) {
                return Err(get_stack_trace_output(
                    "Unexpected error ocurred when running tests.".to_string(),
                    &stack_trace,
                ));
            }
        }

        Ok((result, message, status_code))
    }

    pub fn to_log(&mut self) -> Result<(), String> {
        let logging_container = self.logging_container.clone().unwrap();

        let condition = match &self.value {
            ExpectedValue::LogLevel(level) => {
                logging_container.lock().unwrap().has_log(level.clone())
            }
            _ => {
                return Err("Expected value passed to expect() to be a logging function".to_string())
            }
        };

        if !condition && !self.negative {
            let error = format!("Expected log function to be called but it was not");

            Err(error)
        } else if condition && self.negative {
            let error = format!("Expected log function to not be called but it was");

            Err(error)
        } else {
            Ok(())
        }
    }

    pub fn to_log_message(&mut self, pattern: &str) -> Result<(), String> {
        let logging_container = self.logging_container.clone().unwrap();

        let condition = match &self.value {
            ExpectedValue::LogLevel(level) => logging_container
                .lock()
                .unwrap()
                .has_matching_log(level.clone(), pattern),
            _ => {
                return Err("Expected value passed to expect() to be a logging function".to_string())
            }
        };

        if !condition && !self.negative {
            let logs = logging_container.lock().unwrap().get_logs();
            let error = format!(
                "Expected log function to be called with '{}' but it was not. \n \t\tLogs Captured:\n {}",
                pattern,
                logs.iter().map(|log| format!("\t\t[{}] {}", log.level.to_string(), log.message)).collect::<Vec<_>>().join("\n")
            );

            Err(error)
        } else if condition && self.negative {
            let error = format!(
                "Expected log function to not be called with '{}' but it was",
                pattern
            );

            Err(error)
        } else {
            Ok(())
        }
    }
}
