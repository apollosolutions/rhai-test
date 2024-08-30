use crate::engine::create_engine;
use colored::Colorize;
use regex::Regex;
use rhai::{Dynamic, EvalAltResult, FnPtr, ImmutableString, Map, Position, AST};
use std::fmt::Write;
use std::sync::{Arc, Mutex};

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
}

impl Expector {
    pub fn new(value: Dynamic) -> Self {
        Self {
            value: ExpectedValue::from_dynamic(&value).unwrap(),
            negative: false,
            ast: None,
        }
    }

    pub fn attach_engine_and_ast(&mut self, ast: Arc<Mutex<Option<AST>>>) {
        self.ast = Some(ast);
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

    pub fn to_throw_status(&mut self, status_code_to_match: i64) -> Result<(), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();

        let engine = create_engine();

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        let mut status_code = String::new();

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err);
            status_code = stack_trace.last().unwrap().status_code.clone();

            match **err {
                rhai::EvalAltResult::ErrorInFunctionCall(_, _, ref inner, _) => {
                    if !matches!(**inner, rhai::EvalAltResult::ErrorInFunctionCall(..)) {
                        return Err(get_stack_trace_output(
                            "Unexpected error ocurred when running tests.".to_string(),
                            &stack_trace,
                        ));
                    }
                }
                _ => {
                    return Err(get_stack_trace_output(
                        "Unexpected error ocurred when running tests.".to_string(),
                        &stack_trace,
                    ))
                }
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

        let engine = create_engine();

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        let mut message = String::new();

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err);
            message = stack_trace.last().unwrap().message.clone();

            match **err {
                rhai::EvalAltResult::ErrorInFunctionCall(_, _, ref inner, _) => {
                    if !matches!(**inner, rhai::EvalAltResult::ErrorInFunctionCall(..)) {
                        return Err(get_stack_trace_output(
                            "Unexpected error ocurred when running tests.".to_string(),
                            &stack_trace,
                        ));
                    }
                }
                _ => {
                    return Err(get_stack_trace_output(
                        "Unexpected error ocurred when running tests.".to_string(),
                        &stack_trace,
                    ))
                }
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

        let engine = create_engine();

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        if let Err(ref err) = result {
            let stack_trace = get_stack_trace(err);

            match **err {
                rhai::EvalAltResult::ErrorInFunctionCall(_, _, ref inner, _) => {
                    if !matches!(**inner, rhai::EvalAltResult::ErrorInFunctionCall(..)) {
                        return Err(get_stack_trace_output(
                            "Unexpected error ocurred when running tests.".to_string(),
                            &stack_trace,
                        ));
                    }
                }
                _ => {
                    return Err(get_stack_trace_output(
                        "Unexpected error ocurred when running tests.".to_string(),
                        &stack_trace,
                    ))
                }
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

#[derive(Debug, Clone)]
struct StackTraceDetail {
    pub message: String,
    pub status_code: String,
    pub position: Position,
    pub source: String,
}

impl StackTraceDetail {
    pub fn new(message: String, status_code: String, position: Position, source: String) -> Self {
        Self {
            message,
            status_code,
            position,
            source,
        }
    }
}

fn get_stack_trace(error: &Box<EvalAltResult>) -> Vec<StackTraceDetail> {
    let mut stack_trace = Vec::<StackTraceDetail>::new();

    // TODO: Add rest of arms for error types
    match **error {
        rhai::EvalAltResult::ErrorInFunctionCall(ref name, ref source, ref inner, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                name.clone(),
                "".to_string(),
                position.clone(),
                source.clone(),
            ));

            stack_trace.extend(get_stack_trace(&inner));
        }
        rhai::EvalAltResult::ErrorModuleNotFound(ref module_name, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Module not found: {}. Hint: If you're importing a module in a test file, don't forget to use inline imports scoped to the function you're using the import in.", module_name.clone().to_string()),
                "".to_string(),
                position.clone(),
                "".to_string(),
            ));
        }
        rhai::EvalAltResult::ErrorRuntime(ref error_token, ref position) => {
            if let Some(map) = error_token.read_lock::<Map>() {
                let message = map.get("message").unwrap().to_string();
                let status = map.get("status").unwrap().to_string();

                stack_trace.push(StackTraceDetail::new(
                    message,
                    status.clone().to_string(),
                    position.clone(),
                    "".to_string(),
                ));
            } else {
                stack_trace.push(StackTraceDetail::new(
                    error_token.clone().to_string(),
                    "".to_string(),
                    position.clone(),
                    "".to_string(),
                ));
            }
        }
        _ => {
            println!("\t{}", " Unknown error occurred. ".red());
        }
    }

    stack_trace
}

fn get_stack_trace_output(message: String, stack_trace: &Vec<StackTraceDetail>) -> String {
    let mut output = String::new();

    output.push_str(&message);
    output.push_str("\n");

    // Iterate over stack trace details in reverse order
    for stack_trace_detail in stack_trace.iter().rev() {
        writeln!(
            output,
            "\t\t\tAt {}: {} ({})",
            stack_trace_detail.position, stack_trace_detail.message, stack_trace_detail.source
        )
        .unwrap();
    }

    output
}
