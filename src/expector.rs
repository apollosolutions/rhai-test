use regex::Regex;
use rhai::{Dynamic, EvalAltResult, FnPtr, Func, ImmutableString};

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
}

impl Expector {
    pub fn new(value: Dynamic) -> Self {
        Self {
            value: ExpectedValue::from_dynamic(&value).unwrap(),
            negative: false,
        }
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
        let condition = match &self.value {
            ExpectedValue::Function(value) => value.call((), (), ()),
            _ => return Err("Type mismatch".into()), // TODO: Better message
        };

        Ok(())
    }
}
