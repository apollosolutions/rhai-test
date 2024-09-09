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
        rhai::EvalAltResult::ErrorSystem(ref message, ..) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Unknown System Error: {}", message.clone()),
                "".to_string(),
                Position::NONE,
                "".to_string(),
            ));
        }
        rhai::EvalAltResult::ErrorInFunctionCall(ref name, ref source, ref inner, ref position) => {
            stack_trace.push(StackTraceDetail::new(
                format!("Error in function call: {}", name),
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
        rhai::EvalAltResult::ErrorParsing(ref syntax_error, position) => {
            match syntax_error {
                rhai::ParseErrorType::UnexpectedEOF => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Unexpected end of file",),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::BadInput(ref lex_error) => match lex_error {
                    rhai::LexError::UnexpectedInput(symbol) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: Unexpected symbol: {}", symbol),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::UnterminatedString => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: String literal not terminated before new-line or EOF."),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::StringTooLong(..) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: identifier or string literal longer than the maximum allowed length."),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::MalformedEscapeSequence(sequence) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: string/character/numeric escape sequence is in an invalid format: {}", sequence),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::MalformedNumber(number) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!(
                                "Parsing Error: numeric literal is in an invalid format: {}",
                                number
                            ),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::MalformedChar(char) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!(
                                "Parsing Error: character literal is in an invalid format: {}",
                                char
                            ),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::MalformedIdentifier(identifier) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!(
                                "Parsing Error: identifier is in an invalid format: {}",
                                identifier
                            ),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::ImproperSymbol(a, b) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: Bad symbol encountered: {} {}", a, b),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    rhai::LexError::Runtime(message) => {
                        stack_trace.push(StackTraceDetail::new(
                            format!("Parsing Error: Runtime error: {}", message),
                            "".to_string(),
                            position.clone(),
                            "".to_string(),
                        ));
                    }
                    _ => {
                        println!("\t{}", " Unknown parsing error occurred. ".red());
                    }
                },
                rhai::ParseErrorType::UnknownOperator(ref operator) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: unknown operator encountered: {}", operator),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::MissingToken(ref token, ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Expected token: {} {}", token, description),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::MissingSymbol(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Expected Symbol: {}", description),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::MalformedIndexExpr(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Syntax error with expression in indexing brackets `[]`: {}", description),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::MalformedCapture(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Syntax error with a capture: {}",
                            description
                        ),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::DuplicatedProperty(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Map definition has duplicated property names: {}",
                            description
                        ),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::DuplicatedVariable(ref description) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: variable name duplicated: {}", description),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::WrongSwitchIntegerCase => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: numeric case of `switch` statement is in an appropriate place."),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::WrongSwitchDefaultCase => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: default case of `switch` statement is in an appropriate place."),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::WrongSwitchCaseCondition => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: case condition of `switch` statement is not appropriate"),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::PropertyExpected => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Missing property name for custom type or map"),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::VariableExpected => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Missing variable name after a `let`, `const`, `for` or `catch` keyword."),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::ForbiddenVariable(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Forbidden variable name: {}", name),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::Reserved(name) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Reserved symbol: {}", name),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::MismatchedType(requested, actual) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!(
                            "Parsing Error: Type mismatch. Requested: {}, Actual: {}",
                            requested, actual
                        ),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                rhai::ParseErrorType::ExprExpected(expression) => {
                    stack_trace.push(StackTraceDetail::new(
                        format!("Parsing Error: Expression expected: {}", expression),
                        "".to_string(),
                        position.clone(),
                        "".to_string(),
                    ));
                }
                _ => {
                    println!("\t{}", " Unknown parsing error occurred. ".red());
                }
            }
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
