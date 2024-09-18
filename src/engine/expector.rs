use super::{
    error_handling::{get_inner_most_error, get_stack_trace, get_stack_trace_output},
    logging_container::{LogLevel, LoggingContainer},
    test_container::TestContainer,
};
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

/// Represents all the different types of values that can be passed to an expect() or one of it's functions
#[derive(Debug, Clone)]
pub enum ExpectedValue {
    String(String),
    Bool(bool),
    Int(i64),
    Function(FnPtr),
    Nothing(()),
    Error(String),
    LogLevel(LogLevel),
}

/// This defines how to parse/cast the value into the enum or provide an error that it's an unsupported type
/// When adding new types, make sure to also update the PartialEq definition below so we know how to compare values
impl ExpectedValue {
    pub fn from_dynamic(dynamic: &Dynamic) -> Result<Self, String> {
        if let Some(s) = dynamic.clone().try_cast::<ImmutableString>() {
            Ok(ExpectedValue::String(s.to_string()))
        } else if let Some(b) = dynamic.clone().try_cast::<bool>() {
            Ok(ExpectedValue::Bool(b))
        } else if let Some(i) = dynamic.clone().try_cast::<i64>() {
            Ok(ExpectedValue::Int(i))
        } else if let Some(f) = dynamic.clone().try_cast::<FnPtr>() {
            Ok(ExpectedValue::Function(f))
        } else if let Some(n) = dynamic.clone().try_cast::<()>() {
            Ok(ExpectedValue::Nothing(n))
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

/// Defines how to compare these enum values to each other
impl PartialEq for ExpectedValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExpectedValue::String(s1), ExpectedValue::String(s2)) => s1 == s2,
            (ExpectedValue::Bool(b1), ExpectedValue::Bool(b2)) => b1 == b2,
            (ExpectedValue::Int(i1), ExpectedValue::Int(i2)) => i1 == i2,
            (ExpectedValue::Function(f1), ExpectedValue::Function(f2)) => {
                f1.to_string() == f2.to_string()
            }
            (ExpectedValue::Nothing(n1), ExpectedValue::Nothing(n2)) => n1 == n2,
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
    test_container: Option<Arc<Mutex<TestContainer>>>,
}

impl Expector {
    /// We're going to attempt to parse a provided value into an expector. If it's an invalid value, it'll be given the Error enum type that we'll handle later in the expector functions.
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
            test_container: None,
        }
    }

    /// This attaches all the engine components that the expector needs to evaluate
    pub fn attach(
        &mut self,
        ast: Arc<Mutex<Option<AST>>>,
        test_coverage_container: Arc<Mutex<TestCoverageContainer>>,
        config: Arc<Mutex<Config>>,
        module_cache: Arc<Mutex<BTreeMap<PathBuf, Arc<Module>>>>,
        logging_container: Arc<Mutex<LoggingContainer>>,
        test_container: Arc<Mutex<TestContainer>>,
    ) {
        self.ast = Some(ast);
        self.test_coverage_container = Some(test_coverage_container);
        self.config = Some(config);
        self.module_cache = Some(module_cache);
        self.logging_container = Some(logging_container);
        self.test_container = Some(test_container);
    }

    /// Inverses the check
    pub fn not(mut self) -> Self {
        self.negative = true;
        self
    }

    /// Checks if two values are equal
    pub fn to_be(&mut self, expected: Dynamic) {
        if let ExpectedValue::Error(err_msg) = &self.value {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(err_msg.clone()));
            return ();
        }

        let condition = match &ExpectedValue::from_dynamic(&expected) {
            Ok(val) => &self.value == val,
            Err(error) => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(error.clone()));
                return ();
            }
        };

        if !condition && !self.negative {
            let error = format!(
                "Expected value to be {:?} but instead got {:?}",
                expected, self.value
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!(
                "Expected value {:?} to not be {:?} but it was",
                self.value, expected
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Checks if a value exists (effectively, it's not ())
    pub fn to_exist(&mut self) {
        if let ExpectedValue::Error(err_msg) = &self.value {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(err_msg.clone()));
            return ();
        }

        let condition: bool = if let ExpectedValue::Nothing(_) = &self.value {
            false
        } else {
            true
        };

        if !condition && !self.negative {
            let error = format!("Expected value {:?} to exist", self.value);

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!("Expected value {:?} to not exist", self.value);

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Checks if a provided string matches a provided regular expression
    pub fn to_match(&mut self, pattern: &str) {
        if let ExpectedValue::Error(err_msg) = &self.value {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(err_msg.clone()));
            return ();
        }

        let regex = Regex::new(pattern).unwrap();

        let condition = match &self.value {
            ExpectedValue::String(value) => regex.is_match(value),
            _ => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(
                        "Expected value passed to expect() to be a string".to_string(),
                    ));
                return ();
            }
        };

        if !condition && !self.negative {
            let error = format!(
                "Expected value {:?} to match pattern {:?} but it did not",
                self.value, pattern
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!(
                "Expected value {:?} to not match pattern {:?} but it did",
                self.value, pattern
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Checks if a provided function pointer, when executed, throws a specified status code and/or message
    pub fn to_throw_status_and_message(
        &mut self,
        status_code_to_match: i64,
        message_to_match: &str,
    ) {
        self.to_throw_status(status_code_to_match);
        self.to_throw_message(message_to_match);
    }

    /// Checks if a provided function pointer, when executed, throws a specified status code
    pub fn to_throw_status(&mut self, status_code_to_match: i64) {
        if let ExpectedValue::Error(err_msg) = &self.value {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(err_msg.clone()));
            return ();
        }

        let binding = self.run_throw_function();
        let (result, _, status_code) = match &binding {
            Ok(r) => r,
            Err(error) => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(error.clone()));
                return ();
            }
        };

        let condition = result.is_err();
        let condition2 = status_code.clone() == status_code_to_match.to_string();

        if !condition && !self.negative {
            let error = format!("Expected function to throw but it did not");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!("Expected function to not throw but it did");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && !condition2 {
            let error = format!(
                "Expected function to throw error with status '{}' but instead received '{}'",
                status_code_to_match, status_code
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Checks if a provided function pointer, when executed, throws a specified message
    pub fn to_throw_message(&mut self, message_to_match: &str) {
        if let ExpectedValue::Error(err_msg) = &self.value {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(err_msg.clone()));
            return ();
        }

        let binding = self.run_throw_function();
        let (result, message, _) = match &binding {
            Ok(r) => r,
            Err(error) => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(error.clone()));
                return ();
            }
        };

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

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!("Expected function to not throw but it did");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && (!condition2 && !condition3) {
            let error = format!(
                "Expected function to throw error with message '{}' but instead received '{}'",
                message_to_match, message
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Checks if a provided function pointer, when executed, throws an error
    pub fn to_throw(&mut self) {
        if let ExpectedValue::Error(err_msg) = &self.value {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(err_msg.clone()));
            return ();
        }

        let binding = self.run_throw_function();
        let (result, ..) = match &binding {
            Ok(r) => r,
            Err(error) => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(error.clone()));
                return ();
            }
        };

        let condition = result.is_err();

        if !condition && !self.negative {
            let error = format!("Expected function to throw but it did not");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!("Expected function to not throw but it did");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Executes a function pointer and parses any thrown errors. Used internally by to_throw* functions.
    fn run_throw_function(
        &mut self,
    ) -> Result<(Result<(), Box<EvalAltResult>>, String, String), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();
        let test_coverage_container = self.test_coverage_container.clone().unwrap();
        let config = self.config.clone().unwrap();
        let module_cache = self.module_cache.clone().unwrap();
        let logging_container = self.logging_container.clone().unwrap();

        // Why are we re-creating an engine here? Because the engine is already locked when this function is run, we end up in a thread-lock situation if we try to also use the engine here.
        // So the (unfortunate) solution is to re-create the engine
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

    /// Checks if a given log function has been called during the execution of the current test
    pub fn to_log(&mut self) {
        let logging_container = self.logging_container.clone().unwrap();

        let condition = match &self.value {
            ExpectedValue::LogLevel(level) => {
                logging_container.lock().unwrap().has_log(level.clone())
            }
            _ => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(
                        "Expected value passed to expect() to be a logging function".to_string(),
                    ));
                return ();
            }
        };

        if !condition && !self.negative {
            let error = format!("Expected log function to be called but it was not");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!("Expected log function to not be called but it was");

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }

    /// Checks if a given log function has been called with a particular message (matching a pattern) during the execution of the current test
    /// If this fails, it outputs the logs that it did see to help the user debug
    pub fn to_log_message(&mut self, pattern: &str) {
        let logging_container = self.logging_container.clone().unwrap();

        let condition = match &self.value {
            ExpectedValue::LogLevel(level) => logging_container
                .lock()
                .unwrap()
                .has_matching_log(level.clone(), pattern),
            _ => {
                self.test_container
                    .as_mut()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .add_expect_result(Result::Err(
                        "Expected value passed to expect() to be a logging function".to_string(),
                    ));
                return ();
            }
        };

        if !condition && !self.negative {
            let logs = logging_container.lock().unwrap().get_logs();
            let error = format!(
                "Expected log function to be called with '{}' but it was not. \n \t\tLogs Captured:\n {}",
                pattern,
                logs.iter().map(|log| format!("\t\t[{}] {}", log.level.to_string(), log.message)).collect::<Vec<_>>().join("\n")
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else if condition && self.negative {
            let error = format!(
                "Expected log function to not be called with '{}' but it was",
                pattern
            );

            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Err(error.clone()));
        } else {
            self.test_container
                .as_mut()
                .unwrap()
                .lock()
                .unwrap()
                .add_expect_result(Result::Ok(()));
        }
    }
}
