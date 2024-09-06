use colored::Colorize;
use rhai::{EvalAltResult, Map, Position};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct StackTraceDetail {
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

pub fn get_stack_trace(error: &Box<EvalAltResult>) -> Vec<StackTraceDetail> {
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
        rhai::EvalAltResult::ErrorFunctionNotFound(ref function_signature, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!(
                    "Function not found: {}.",
                    function_signature.clone().to_string()
                ),
                "".to_string(),
                position.clone(),
                "".to_string(),
            ));
        }
        _ => {
            println!("\t{}", " Unknown error occurred. ".red());
        }
    }

    stack_trace
}

pub fn get_stack_trace_output(message: String, stack_trace: &Vec<StackTraceDetail>) -> String {
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

pub fn get_inner_most_error(error: &Box<EvalAltResult>) -> &Box<EvalAltResult> {
    let inner_most_error;

    match **error {
        rhai::EvalAltResult::ErrorInFunctionCall(_, _, ref inner, _) => {
            inner_most_error = get_inner_most_error(inner);
        }
        _ => {
            inner_most_error = error;
        }
    }

    inner_most_error
}
