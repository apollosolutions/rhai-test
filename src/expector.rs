use crate::engine::create_engine;
use regex::Regex;
use rhai::{Dynamic, EvalAltResult, FnPtr, ImmutableString, AST};
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

    pub fn to_throw(&mut self) -> Result<(), String> {
        let ast_guard = &self.ast.as_ref().unwrap().lock().unwrap();
        let ast = ast_guard.as_ref().unwrap();

        let engine = create_engine();

        let result = match &self.value {
            ExpectedValue::Function(value) => value.call::<()>(&engine, ast, ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        // TODO: Capture error message(s) (stack track type of thing?) and display it on bad errors... effectively do a recursive function that goes through the errors and captures the message and adds it to a vec
        // TODO: Capture inner error message on good errors so we can do tests against it
        // TODO: Can we do helpful error messages for certain error types? (E.g. ErrorModuleNotFound remind the user to import into inner scope?)
        if let Err(ref err) = result {
            match **err {
                rhai::EvalAltResult::ErrorInFunctionCall(_, _, ref inner, _) => {
                    if !matches!(**inner, rhai::EvalAltResult::ErrorInFunctionCall(..)) {
                        // This runs if the inner error is NOT an ErrorInFunctionCall
                        return Err("Unexpected error ocurred when running tests.".to_string());
                    }
                }
                _ => return Err("Unexpected error ocurred when running tests.".to_string()),
            }
        }

        let condition = result.is_err();

        // TODO: Support specific throw messages
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
